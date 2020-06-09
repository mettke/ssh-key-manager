use core_common::{
    chrono::offset::Utc,
    database::{Create, Database, DatabaseError, DbList, FetchAll, FetchById},
    objects::{Entity, Group, GroupFilter, GroupMember, User},
    sec::{Auth, CsrfToken},
    serde::Serialize,
    types::Id,
    web::{AppError, Notification, Request, TemplateEngine},
};
use std::borrow::Cow;

/// A List of groups ready to be presented
#[derive(Debug)]
pub struct GroupListView<'a>(pub DbList<Group<'a>>);

impl<'a> GroupListView<'a> {
    /// Fetches all groups visible to the given user
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, single_use_lifetimes)]
    pub async fn fetch<A, D, T, R>(
        req: &R,
        filter: &GroupFilter<'_>,
    ) -> Result<GroupListView<'a>, AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b, 'c> D: Database + FetchAll<'b, A, Group<'a>, GroupFilter<'c>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let auth = req.get_auth();
        let db = req.get_database();

        db.fetch_all(filter, auth, 0)
            .map(Self)
            .map_err(AppError::DatabaseError)
    }

    /// Creates a `Group` using the information in the request body
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, single_use_lifetimes, clippy::needless_lifetimes)]
    pub async fn create<'e, A, D, T, R>(
        req: &mut R,
        name: Option<Cow<'e, str>>,
        oauth_scope: Option<Cow<'e, str>>,
        ldap_group: Option<Cow<'e, str>>,
        csrf: &CsrfToken,
    ) -> Result<[Notification<'e>; 1], AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b, 'c, 'd> D: Database
            + FetchAll<'b, A, Group<'d>, GroupFilter<'c>, D>
            + Create<A, Group<'d>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        if !csrf.valid {
            return Ok([Notification::Error {
                name: "Group",
                para: "csrf",
                help: "../help/#group_err",
            }]);
        }
        let db = req.get_database();
        let auth = req.get_auth();
        let id = db.generate_id()?;
        if let Some(name) = name {
            let group = Group {
                entity_id: Cow::Owned(id),
                name,
                system: false,
                oauth_scope,
                ldap_group,
            };
            match db.create(&group, auth) {
                Err(err @ DatabaseError::Custom(_)) => {
                    Err(AppError::DatabaseError(err))
                }
                Err(DatabaseError::NonUnique) => Ok([Notification::Unique {
                    name: "Group",
                    para: "name",
                    help: "../help/#group_err",
                }]),
                Ok(()) => Ok([Notification::Info {
                    name: "Group",
                    url: ".",
                    id: group.entity_id,
                }]),
            }
        } else {
            Ok([Notification::Error {
                name: "Group",
                para: "Group Name",
                help: "../help/#group_err",
            }])
        }
    }
}

/// A group ready to be presented
#[derive(Debug, Clone, Hash, Serialize)]
pub struct GroupView<'a> {
    /// The group to show to the user
    pub group: Group<'a>,
    /// Enities which are members of the group
    pub members: Vec<GroupMember<'a, Entity<'a>>>,
    /// Whether the current user is the owner
    pub is_member: bool,
}

#[allow(clippy::unimplemented)]
impl GroupView<'_> {
    /// Fetches all groups visible to the given user
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, single_use_lifetimes)]
    pub async fn fetch<'a, A, D, T, R>(
        req: &R,
        id: &str,
    ) -> Result<Option<GroupView<'a>>, AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b> D: Database
            + FetchById<'b, A, Group<'a>, D>
            + FetchAll<'b, A, GroupMember<'a, Entity<'a>>, Id, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        let auth = req.get_auth();
        let db = req.get_database();

        let id = match Id::from_string(id) {
            Err(_) => {
                return Ok(None);
            }
            Ok(id) => id,
        };

        let group: Option<Group<'_>> = db.fetch(&id, auth)?;
        if let Some(group) = group {
            let members: Vec<GroupMember<'a, Entity<'a>>> =
                db.fetch_all(&group.entity_id, auth, 0)?.data;
            Ok(Some(GroupView {
                group,
                members,
                is_member: false,
            }))
        } else {
            Ok(None)
        }
    }

    /// Creates a `Group` using the information in the request body
    ///
    /// # Errors
    /// Fails when database connection fails
    #[inline]
    #[allow(unused_lifetimes, single_use_lifetimes, clippy::needless_lifetimes)]
    pub async fn add_member_user<'e, A, D, T, R>(
        req: &mut R,
        id: Id,
        uid: Option<&User<'_>>,
        csrf: &CsrfToken,
    ) -> Result<[Notification<'e>; 1], AppError<A, D, T, R>>
    where
        A: Auth,
        for<'b, 'c, 'd> D: Database
            + FetchAll<'b, A, Group<'d>, GroupFilter<'c>, D>
            + Create<A, GroupMember<'d, Cow<'d, Id>>, D>,
        T: TemplateEngine,
        R: Request<A, D, T>,
    {
        if !csrf.valid {
            return Ok([Notification::Error {
                name: "Group",
                para: "csrf",
                help: "../../help/#group_err",
            }]);
        }
        let db = req.get_database();
        let auth = req.get_auth();
        if let Some(user) = uid {
            let add_date = Utc::now().naive_utc();
            let member = GroupMember {
                group_id: Cow::Owned(id),
                member: user.entity_id.clone(),
                add_date,
            };
            match db.create(&member, auth) {
                Err(err @ DatabaseError::Custom(_)) => {
                    Err(AppError::DatabaseError(err))
                }
                Err(DatabaseError::NonUnique) => Ok([Notification::Unique {
                    name: "Group",
                    para: "Username",
                    help: "../help/#group_err",
                }]),
                Ok(()) => Ok([Notification::Modified { name: "Group" }]),
            }
        } else {
            Ok([Notification::Error {
                name: "Group",
                para: "Username",
                help: "../help/#group_err",
            }])
        }
    }
}
