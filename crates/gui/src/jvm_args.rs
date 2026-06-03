#[derive(Debug, Clone)]
pub struct JvmArgs {
    pub max_memory_mb: u32,
    pub min_memory_mb: u32,
    pub gc_type: GcType,
    pub custom_flags: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum GcType {
    G1GC,
    ZGC,
    Shenandoah,
    Serial,
    Parallel,
}

impl Default for JvmArgs {
    fn default() -> Self {
        Self {
            max_memory_mb: 2048,
            min_memory_mb: 512,
            gc_type: GcType::G1GC,
            custom_flags: vec!["-XX:+UnlockExperimentalVMOptions".to_string()],
        }
    }
}

impl JvmArgs {
    pub fn build_command_args(&self) -> Vec<String> {
        let mut args = vec![
            format!("-Xmx{}M", self.max_memory_mb),
            format!("-Xms{}M", self.min_memory_mb),
        ];
        
        let gc_flag = match self.gc_type {
            GcType::G1GC => "-XX:+UseG1GC",
            GcType::ZGC => "-XX:+UseZGC",
            GcType::Shenandoah => "-XX:+UseShenandoahGC",
            GcType::Serial => "-XX:+UseSerialGC",
            GcType::Parallel => "-XX:+UseParallelGC",
        };
        args.push(gc_flag.to_string());
        
        args.extend(self.custom_flags.iter().cloned());
        args
    }
}
