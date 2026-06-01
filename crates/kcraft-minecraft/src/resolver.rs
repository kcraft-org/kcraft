use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId(pub String);

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub id: PackageId,
    pub version: String,
    pub requires: Vec<PackageId>,
    pub conflicts: Vec<PackageId>,
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error("Missing dependency {1:?} required by {0:?}")]
    MissingDependency(PackageId, PackageId),
    #[error("Conflict detected between {0:?} and {1:?}")]
    Conflict(PackageId, PackageId),
    #[error("Cycle detected involving {0:?}")]
    CycleDetected(PackageId),
}

/// A lightweight DAG-based Dependency Resolver and constraint satisfaction verifier
#[derive(Default)]
pub struct Resolver {
    nodes: HashMap<PackageId, DependencyNode>,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: DependencyNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Resolves the dependency graph starting from the given roots,
    /// returning a topologically sorted order of packages to install.
    /// Returns an error if a cycle, missing dependency, or conflict is detected.
    pub fn resolve(&self, roots: &[PackageId]) -> Result<Vec<PackageId>, ResolverError> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();

        for root in roots {
            self.visit(root, &mut visited, &mut stack, &mut resolved)?;
        }

        // Validate conflicts globally across the resolved subset
        let resolved_set: HashSet<_> = resolved.iter().cloned().collect();
        for id in &resolved {
            if let Some(node) = self.nodes.get(id) {
                for conflict in &node.conflicts {
                    if resolved_set.contains(conflict) {
                        return Err(ResolverError::Conflict(id.clone(), conflict.clone()));
                    }
                }
            }
        }

        Ok(resolved)
    }

    fn visit(
        &self,
        id: &PackageId,
        visited: &mut HashSet<PackageId>,
        stack: &mut HashSet<PackageId>,
        resolved: &mut Vec<PackageId>,
    ) -> Result<(), ResolverError> {
        if visited.contains(id) {
            return Ok(());
        }
        if stack.contains(id) {
            return Err(ResolverError::CycleDetected(id.clone()));
        }

        let node = self.nodes.get(id).ok_or_else(|| {
            ResolverError::MissingDependency(PackageId("root".into()), id.clone())
        })?;

        stack.insert(id.clone());

        for req in &node.requires {
            self.visit(req, visited, stack, resolved).map_err(|e| {
                if let ResolverError::MissingDependency(ref root, ref missing) = e {
                    if root.0 == "root" {
                        ResolverError::MissingDependency(id.clone(), missing.clone())
                    } else {
                        e
                    }
                } else {
                    e
                }
            })?;
        }

        stack.remove(id);
        visited.insert(id.clone());
        resolved.push(id.clone());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolver_basic_dag() {
        let mut resolver = Resolver::new();
        resolver.add_node(DependencyNode {
            id: PackageId("A".into()),
            version: "1.0".into(),
            requires: vec![PackageId("B".into()), PackageId("C".into())],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("B".into()),
            version: "1.0".into(),
            requires: vec![PackageId("D".into())],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("C".into()),
            version: "1.0".into(),
            requires: vec![PackageId("D".into())],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("D".into()),
            version: "1.0".into(),
            requires: vec![],
            conflicts: vec![],
        });

        let resolved = resolver.resolve(&[PackageId("A".into())]).unwrap();
        // D should be resolved first, then B or C, then A
        assert_eq!(resolved[0], PackageId("D".into()));
        assert_eq!(resolved.last().unwrap(), &PackageId("A".into()));
        assert_eq!(resolved.len(), 4);
    }

    #[test]
    fn test_resolver_conflict() {
        let mut resolver = Resolver::new();
        resolver.add_node(DependencyNode {
            id: PackageId("A".into()),
            version: "1.0".into(),
            requires: vec![PackageId("B".into()), PackageId("C".into())],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("B".into()),
            version: "1.0".into(),
            requires: vec![],
            conflicts: vec![PackageId("C".into())],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("C".into()),
            version: "1.0".into(),
            requires: vec![],
            conflicts: vec![],
        });

        let res = resolver.resolve(&[PackageId("A".into())]);
        assert!(matches!(res, Err(ResolverError::Conflict(_, _))));
    }

    #[test]
    fn test_resolver_cycle() {
        let mut resolver = Resolver::new();
        resolver.add_node(DependencyNode {
            id: PackageId("A".into()),
            version: "1.0".into(),
            requires: vec![PackageId("B".into())],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("B".into()),
            version: "1.0".into(),
            requires: vec![PackageId("C".into())],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("C".into()),
            version: "1.0".into(),
            requires: vec![PackageId("A".into())],
            conflicts: vec![],
        });

        let res = resolver.resolve(&[PackageId("A".into())]);
        assert!(matches!(res, Err(ResolverError::CycleDetected(_))));
    }

    #[test]
    fn test_resolver_dependency_ordering() {
        let mut resolver = Resolver::new();
        resolver.add_node(DependencyNode {
            id: PackageId("mod_a".into()),
            version: "1.0".into(),
            requires: vec![],
            conflicts: vec![],
        });
        resolver.add_node(DependencyNode {
            id: PackageId("mod_b".into()),
            version: "1.0".into(),
            requires: vec![PackageId("mod_a".into())],
            conflicts: vec![],
        });
        let resolved = resolver.resolve(&[PackageId("mod_b".into())]).unwrap();
        let mod_a_pos = resolved.iter().position(|d| d.0 == "mod_a");
        let mod_b_pos = resolved.iter().position(|d| d.0 == "mod_b");
        assert!(mod_a_pos < mod_b_pos, "mod_a must come before mod_b");
    }
}
