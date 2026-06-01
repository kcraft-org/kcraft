use kcraft_auth::{generate_offline_uuid, AccountList, MinecraftAccount};
use kcraft_core::account::{AccountData, AccountType, MinecraftProfile};

#[test]
fn test_account_list_save_load_roundtrip() {
    let dir = std::env::temp_dir().join("kcraft_test_auth_roundtrip");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("accounts.json");

    {
        let mut list = AccountList::new(path.clone());
        list.set_autosave(false);

        let profile = MinecraftProfile {
            name: "TestPlayer".to_string(),
            id: generate_offline_uuid("TestPlayer"),
            ..Default::default()
        };
        let data = AccountData {
            account_type: AccountType::Offline,
            minecraft_profile: profile,
            ..Default::default()
        };
        list.add_account(MinecraftAccount { data, active: true });
        list.save().unwrap();
    }

    {
        let list = AccountList::new(path.clone());
        assert_eq!(list.count(), 1);
        if let Some(acc) = list.at(0) {
            assert_eq!(acc.data.minecraft_profile.name, "TestPlayer");
            assert_eq!(acc.data.account_type, AccountType::Offline);
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_multiple_accounts() {
    let dir = std::env::temp_dir().join("kcraft_test_multi_account");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("accounts.json");

    {
        let mut list = AccountList::new(path.clone());
        list.set_autosave(false);

        for i in 0..3 {
            let name = format!("Player{}", i);
            let profile = MinecraftProfile {
                name: name.clone(),
                id: generate_offline_uuid(&name),
                ..Default::default()
            };
            let data = AccountData {
                account_type: AccountType::Offline,
                minecraft_profile: profile,
                ..Default::default()
            };
            list.add_account(MinecraftAccount {
                data,
                active: i == 0,
            });
        }
        list.save().unwrap();
    }

    {
        let list = AccountList::new(path.clone());
        assert_eq!(list.count(), 3);
        let default = list.default_account();
        assert!(default.is_some());
        if let Some(acc) = default {
            assert_eq!(acc.data.minecraft_profile.name, "Player0");
        }
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_generate_offline_uuid_consistency() {
    let uuid1 = generate_offline_uuid("Player");
    let uuid2 = generate_offline_uuid("Player");
    let uuid3 = generate_offline_uuid("Different");

    assert_eq!(uuid1, uuid2, "Same username must produce same UUID");
    assert_ne!(
        uuid1, uuid3,
        "Different usernames must produce different UUIDs"
    );
    assert_eq!(uuid1.len(), 36, "UUID must be 36 chars with dashes");
    assert_eq!(
        uuid1.chars().filter(|&c| c == '-').count(),
        4,
        "UUID must have 4 dashes"
    );
}
