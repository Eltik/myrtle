#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

use myrtle_app::policy::{Grant, Permission};
use myrtle_app::{Action, AppError, Decision, Policy, Resource, Subject};
use myrtle_types::{ApiKeyId, Role, Server, TierListId, TierListType, UserId, WorkerId};
use uuid::Uuid;

const ROLES: [Role; 5] = [
    Role::User,
    Role::Translator,
    Role::TierListEditor,
    Role::TierListAdmin,
    Role::SuperAdmin,
];

const fn id(n: u128) -> UserId {
    UserId(Uuid::from_u128(n))
}

fn user(n: u128, role: Role) -> Subject {
    granted(n, role, Vec::new())
}

fn granted(n: u128, role: Role, grants: Vec<Grant>) -> Subject {
    Subject::User {
        id: id(n),
        uid: n.to_string().into(),
        server: Some(Server::EN),
        role,
        grants: grants.into(),
    }
}

fn can(subject: &Subject, action: Action, resource: &Resource) -> bool {
    Policy::default()
        .can(subject, action, resource)
        .is_allowed()
}

fn list(n: u128, owner: Option<u128>) -> Resource {
    Resource::TierList {
        id: TierListId(Uuid::from_u128(n)),
        owner: owner.map(id),
    }
}

fn list_grant(permission: Permission, n: u128) -> Grant {
    Grant {
        action: permission.tier_list_action(),
        resource_id: Some(Uuid::from_u128(n).to_string()),
    }
}

const TIER_ACTIONS: [Action; 4] = [
    Action::ViewTierList,
    Action::EditTierList,
    Action::PublishTierList,
    Action::AdministerTierList,
];

#[test]
fn role_gates_match_the_role_predicates() {
    for role in ROLES {
        let staff = matches!(
            role,
            Role::TierListEditor | Role::TierListAdmin | Role::SuperAdmin
        );
        let list_admin = matches!(role, Role::TierListAdmin | Role::SuperAdmin);
        let super_admin = role == Role::SuperAdmin;
        let subject = user(1, role);
        let expect = |action, resource: Resource, allowed: bool| {
            assert_eq!(
                can(&subject, action, &resource),
                allowed,
                "{role:?} {action:?}"
            );
        };

        expect(
            Action::AccessAdminPanel,
            Resource::AdminPanel,
            role != Role::User,
        );
        expect(Action::EditOperatorNotes, Resource::OperatorNotes, staff);
        expect(
            Action::CreateTierList,
            Resource::NewTierList {
                kind: TierListType::Official,
            },
            staff,
        );
        expect(
            Action::CreateTierList,
            Resource::NewTierList {
                kind: TierListType::Community,
            },
            true,
        );
        for action in [
            Action::ViewTierListAdminStats,
            Action::ListUsers,
            Action::ViewSiteStats,
            Action::ReadOthersNoteAudit,
            Action::ReadAudit,
        ] {
            expect(action, Resource::Site, list_admin);
        }
        expect(Action::ManageTierListGrants, list(9, None), list_admin);
        expect(
            Action::EditGrid,
            Resource::Grid { owner: id(2) },
            list_admin,
        );
        for action in TIER_ACTIONS {
            expect(action, list(9, Some(2)), list_admin);
        }
        for action in [
            Action::SetUserRole,
            Action::ManageLocales,
            Action::EditReleaseData,
            Action::ManageApiKeys,
        ] {
            expect(action, Resource::Site, super_admin);
        }
        let locale = Resource::Locale { code: "fr".into() };
        expect(Action::WriteTranslations, locale.clone(), super_admin);
        expect(Action::ManageTranslators, locale, super_admin);
        expect(Action::BypassRateLimit, Resource::Site, false);
        expect(Action::LeaseJobs, Resource::JobQueue, false);
    }
}

#[test]
fn an_owner_holds_every_level_on_their_own_list_and_grid() {
    let owner = user(1, Role::User);
    for action in TIER_ACTIONS {
        assert!(can(&owner, action, &list(9, Some(1))), "{action:?}");
    }
    assert!(can(
        &owner,
        Action::EditGrid,
        &Resource::Grid { owner: id(1) }
    ));
}

#[test]
fn a_list_grant_covers_its_level_and_below_on_that_list_only() {
    for (held, permission) in Permission::ALL.into_iter().enumerate() {
        let subject = granted(1, Role::TierListEditor, vec![list_grant(permission, 9)]);
        for (asked, action) in TIER_ACTIONS.into_iter().enumerate() {
            assert_eq!(
                can(&subject, action, &list(9, Some(2))),
                asked <= held,
                "{permission:?} for {action:?}"
            );
            assert!(!can(&subject, action, &list(8, Some(2))), "another list");
        }
    }
}

#[test]
fn a_locale_grant_writes_that_locale_and_opens_the_admin_panel() {
    let grant = |permission: Permission| Grant {
        action: permission.translation_action(),
        resource_id: Some("fr".into()),
    };
    let fr = Resource::Locale { code: "fr".into() };
    let de = Resource::Locale { code: "de".into() };

    let editor = granted(1, Role::User, vec![grant(Permission::Edit)]);
    assert!(can(&editor, Action::WriteTranslations, &fr));
    assert!(!can(&editor, Action::ManageTranslators, &fr));
    assert!(!can(&editor, Action::WriteTranslations, &de));
    assert!(can(
        &editor,
        Action::AccessAdminPanel,
        &Resource::AdminPanel
    ));

    let admin = granted(1, Role::User, vec![grant(Permission::Admin)]);
    assert!(can(&admin, Action::WriteTranslations, &fr));
    assert!(can(&admin, Action::ManageTranslators, &fr));

    let viewer = granted(1, Role::User, vec![grant(Permission::View)]);
    assert!(can(
        &viewer,
        Action::AccessAdminPanel,
        &Resource::AdminPanel
    ));
    assert!(!can(&viewer, Action::WriteTranslations, &fr));

    let publisher = granted(1, Role::User, vec![grant(Permission::Publish)]);
    assert!(can(&publisher, Action::WriteTranslations, &fr));
    assert!(!can(&publisher, Action::ManageTranslators, &fr));

    let lists_only = granted(1, Role::User, vec![list_grant(Permission::Admin, 9)]);
    assert!(!can(
        &lists_only,
        Action::AccessAdminPanel,
        &Resource::AdminPanel
    ));
    assert!(!can(&lists_only, Action::WriteTranslations, &fr));
}

#[test]
fn a_private_profile_is_its_owners_alone() {
    let policy = Policy::default();
    let profile = |public| Resource::Profile {
        owner: id(1),
        public,
    };
    let tab = |visible| Resource::ProfileTab {
        owner: id(1),
        tab: myrtle_types::ProfileTab::Roster,
        visible,
    };
    let visitor = user(2, Role::SuperAdmin);

    for subject in [&Subject::Anonymous, &visitor, &user(1, Role::User)] {
        assert!(can(subject, Action::ReadProfile, &profile(true)));
        assert!(can(subject, Action::ReadProfileTab, &tab(true)));
    }
    assert!(can(
        &user(1, Role::User),
        Action::ReadProfile,
        &profile(false)
    ));
    assert!(can(
        &user(1, Role::User),
        Action::ReadProfileTab,
        &tab(false)
    ));
    assert!(matches!(
        policy.require(&visitor, Action::ReadProfile, &profile(false)),
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        policy.require(&visitor, Action::ReadProfileTab, &tab(false)),
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        policy.require(&Subject::Anonymous, Action::ReadProfile, &profile(false)),
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        policy.require(&Subject::Anonymous, Action::ReadProfileTab, &tab(false)),
        Err(AppError::Forbidden)
    ));
    let grid = Resource::Grid { owner: id(1) };
    assert_eq!(
        policy.can(&Subject::Anonymous, Action::EditGrid, &grid),
        Decision::Deny {
            reason: "sign in required"
        }
    );
    assert!(matches!(
        policy.require(&Subject::Anonymous, Action::EditGrid, &grid),
        Err(AppError::Unauthorized)
    ));
}

#[test]
fn non_user_subjects_hold_exactly_what_they_are_given() {
    let key = Subject::ApiKey {
        id: ApiKeyId(Uuid::from_u128(7)),
        scopes: ["bypass_rate_limit".to_owned(), "lease_jobs".to_owned()].into(),
        owner: None,
    };
    let worker = Subject::Worker {
        id: WorkerId(Uuid::from_u128(8)),
    };
    for action in Action::ALL {
        let scoped = matches!(action, Action::BypassRateLimit | Action::LeaseJobs);
        assert_eq!(can(&key, action, &Resource::Site), scoped, "{action:?}");
        assert_eq!(
            can(&worker, action, &Resource::JobQueue),
            action == Action::LeaseJobs,
            "{action:?}"
        );
        assert!(can(&Subject::System, action, &Resource::Site));
    }
}

#[test]
fn action_scopes_are_unique_and_parse_back() {
    let names: BTreeSet<&str> = Action::ALL.iter().map(|a| a.as_str()).collect();
    assert_eq!(names.len(), Action::ALL.len());
    for action in Action::ALL {
        assert_eq!(Action::parse(action.as_str()), Some(action));
    }
    for permission in Permission::ALL {
        assert_eq!(Permission::parse(permission.as_str()), Some(permission));
    }
    assert_eq!(Permission::parse("ADMIN"), Some(Permission::Admin));
    assert_eq!(Permission::parse("owner"), None);
}
