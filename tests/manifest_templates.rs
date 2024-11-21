use std::mem;

// INSTANTIATE
use flex_pool_test_helper::*;
use radix_transactions::model::InstructionV1;
use scrypto::prelude::*;
use scrypto_test::utils::dump_manifest_to_file_system;

#[test]
fn test_dump_instantiate() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate(
        helper.x_address(),
        helper.y_address(),
        dec!(0),
        dec!(0.5),
        helper.registry.registry_address.unwrap(),
    );
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("instantiate"),
        &NetworkDefinition::simulator(),
    )
    .err();
}

#[test]
fn test_dump_instantiate_with_liquidity() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper
        .registry
        .instantiate_default(helper.registry.admin_badge_address());
    helper.instantiate_with_liquidity(
        helper.x_address(),
        dec!(20),
        helper.y_address(),
        dec!(30),
        dec!(0),
        helper.registry.registry_address.unwrap(),
    );
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("instantiate_with_liquidity"),
        &NetworkDefinition::simulator(),
    )
    .err();
}

#[test]
fn test_dump_add_liquidity() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper.add_liquidity_default(dec!(20), dec!(30));
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("add_liquidity"),
        &NetworkDefinition::simulator(),
    )
    .err();
}

#[test]
fn test_dump_swap() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper.swap(helper.x_address(), dec!(5));
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("swap"),
        &NetworkDefinition::simulator(),
    )
    .err();
}

#[test]
fn test_dump_remove_liquidity() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.instantiate_default(true);
    helper.remove_liquidity_default(dec!(3));
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("remove_liquidity"),
        &NetworkDefinition::simulator(),
    )
    .err();
}

#[test]
#[ignore = "Only run manually, due to unsupported OwnerRole::Fixed and OwnerRole::Updatable"]
fn test_create_token() {
    /*
    Stokenet:
        RESOURCE_PACKAGE = package_tdx_2_1pkgxxxxxxxxxresrcexxxxxxxxx000538436477xxxxxxxxxmn4mes
    Mainnet:
        RESOURCE_PACKAGE = package_rdx1pkgxxxxxxxxxresrcexxxxxxxxx000538436477xxxxxxxxxresrce
    */
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .allocate_global_address(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
            "owner_address_reservation",
            "owner_address",
        )
        .with_name_lookup(|builder, lookup| {
            let owner_address_reservation = lookup.address_reservation("owner_address_reservation");
            let (manifest, _) = builder.add_instruction_advanced(InstructionV1::CallFunction {
                package_address: RESOURCE_PACKAGE.into(),
                blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_string(),
                function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT
                    .to_string(),
                args: to_manifest_value_and_unwrap!(
                    &(FungibleResourceManagerCreateWithInitialSupplyManifestInput {
                        /*
                        Setting the fixed owner role currently doesn't work. As a workaround we can specifiy it manually in the manifest via:
                        Enum<OwnerRole::Fixed>(
                            Enum<AccessRule::Protected>(
                                Enum<AccessRuleNode::ProofRule>(
                                    Enum<ProofRule::Require>(
                                        Enum<ResourceOrNonFungible::Resource>(
                                            NamedAddress("owner_address")
                                        )
                                    )
                                )
                            )
                        )
                         */
                        owner_role: OwnerRole::None,
                        divisibility: 1,
                        track_total_supply: false,
                        metadata: metadata!(
                            init {
                                "name" => "OCI Owner Badge".to_owned(), locked;
                            }
                        ),
                        resource_roles: FungibleResourceRoles {
                            mint_roles: mint_roles! {
                                minter => None;
                                minter_updater => None;
                            },
                            burn_roles: burn_roles! {
                                burner => None;
                                burner_updater => None;
                            },
                            ..Default::default()
                        },
                        initial_supply: dec!(1),
                        address_reservation: Some(owner_address_reservation),
                    })
                ),
            });
            manifest
        });
    let manifest_builder = manifest_builder
        .with_name_lookup(|builder, _| {
            builder.create_fungible_resource(
                /*
                Enum<OwnerRole::Updatable>(
                    Enum<AccessRule::Protected>(
                        Enum<AccessRuleNode::ProofRule>(
                            Enum<ProofRule::Require>(
                                Enum<ResourceOrNonFungible::Resource>(
                                    NamedAddress("owner_address")
                                )
                            )
                        )
                    )
                )
                */
                OwnerRole::None,
                false,
                18,
                Default::default(),
                metadata! {
                    init {
                        "name" => "Ociswap".to_owned(), locked;
                        "symbol" => "OCI".to_owned(), locked;
                        "dapp_definitions" => vec![helper.registry.env.dapp_definition], updatable;
                        "description" => "Ociswap is the Front Page of Radix".to_owned(), updatable;
                        "tags" => vec!["defi", "dex"], updatable;
                        "icon_url" => Url::of("https://ociswap.com/icons/oci.png"), updatable;
                        "info_url" => Url::of("https://ociswap.com"), updatable;
                    }
                },
                Some((100_000_000u64).into()),
            )
        })
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("create_token"),
        &NetworkDefinition::simulator(),
    )
    .err();
}

#[test]
fn test_dump_set_whitelist() {
    let mut helper: FlexPoolTestHelper = FlexPoolTestHelper::new();
    helper.set_whitelist_registry();
    helper.lock_whitelist_registry();
    helper.set_whitelist_hook_value(Vec::<GlobalAddress>::new());
    // just use registry as dummy package for hook, since it is already loaded in the environment
    helper.set_whitelist_hook("registry");
    helper.lock_whitelist_hook();
    let manifest_builder = mem::take(&mut helper.registry.env.manifest_builder)
        .deposit_batch(helper.registry.env.account);
    dump_manifest_to_file_system(
        manifest_builder.object_names(),
        &manifest_builder.build(),
        "./transaction-manifest",
        Some("whitelist"),
        &NetworkDefinition::simulator(),
    )
    .err();
}
