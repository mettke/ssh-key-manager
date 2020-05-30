use crate::{
    database::{Create, Database, DbResult, FetchByUid, Save},
    sec::Auth,
    serde::Serialize,
    types::{Id, UserTypes},
};
use std::{borrow::Cow, convert::TryFrom};

#[derive(Debug, Clone, Hash, Serialize)]
/// Defines the User structure in the database
pub struct User<'a> {
    /// The id which uniquely identifies the user
    pub entity_id: Cow<'a, Id>,
    /// The uid of the user. Must be unique
    pub uid: Cow<'a, str>,
    /// The name of the user
    pub name: Option<Cow<'a, str>>,
    /// The email of the user
    pub email: Option<Cow<'a, str>>,
    /// The password of the user
    pub password: Option<Cow<'a, str>>,
    /// The type of the user
    pub type_: UserTypes,
}

impl<'a> User<'a> {
    /// Checks whether a user already exists and updated it if necessary.
    /// If the user does not exist, it will be created and returned
    ///
    /// # Errors
    /// Fails only on connection or deserialisation errors.
    /// Does not fail on input errors.
    #[inline]
    pub fn update_or_create_user<D, A>(
        db: &D,
        auth: &A,
        uid: &'a str,
        name: &'a str,
        email: &'a str,
        type_: UserTypes,
    ) -> DbResult<Self, D>
    where
        A: Auth,
        D: Database
            + FetchByUid<A, User<'a>, D>
            + Create<A, User<'a>, D>
            + Save<A, User<'a>, D>,
    {
        if let Some(mut user) = db.fetch_by_uid(uid, auth)? {
            if user.requires_update(name, email, type_) {
                db.save(&user, auth)?;
            }
            Ok(user)
        } else {
            let id = db.generate_id()?;
            let user = User {
                entity_id: Cow::Owned(id),
                uid: Cow::Borrowed(uid),
                name: Some(Cow::Borrowed(name)),
                email: Some(Cow::Borrowed(email)),
                password: None,
                type_,
            };
            db.create(&user, auth)?;
            Ok(user)
        }
    }

    #[allow(clippy::useless_let_if_seq)]
    fn requires_update(
        &mut self,
        name: &'a str,
        email: &'a str,
        type_: UserTypes,
    ) -> bool {
        let mut update_required = false;
        if self.name.as_ref().map(AsRef::as_ref) != Some(name) {
            self.name = Some(name.into());
            update_required = true;
        }
        if self.email.as_ref().map(AsRef::as_ref) != Some(email) {
            self.email = Some(email.into());
            update_required = true;
        };
        if self.type_ != type_ {
            self.type_ = type_;
            update_required = true;
        }
        update_required
    }
}

#[derive(Debug, Clone, Hash, Serialize)]
/// Provides fields to filter when searching for multiple
/// objects
pub struct UserFilter<'a> {
    /// The name of the user must be like this value
    pub uid: Option<Cow<'a, str>>,
    /// The name of the user must be like this value
    pub name: Option<Cow<'a, str>>,
    /// The email of the user must be like this value
    pub email: Option<Cow<'a, str>>,
    /// The type of the user must be equal to any of these values
    pub type_: Option<Cow<'a, [UserTypes]>>,
}

impl Default for UserFilter<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            uid: None,
            name: None,
            email: None,
            type_: None,
        }
    }
}

impl<'a, I: Iterator<Item = (Cow<'a, str>, Cow<'a, str>)>> From<I>
    for UserFilter<'a>
{
    #[inline]
    fn from(iter: I) -> Self {
        let mut filter = Self::default();
        let mut types = vec![];
        for (key, val) in iter {
            if val.is_empty() {
                continue;
            }
            match key.as_ref() {
                "uid" => {
                    filter.uid = Some(val);
                }
                "name" => {
                    filter.name = Some(val);
                }
                "email" => {
                    filter.email = Some(val);
                }
                "type" => {
                    if let Ok(v) = UserTypes::try_from(val.as_ref()) {
                        types.push(v);
                    }
                }
                _ => {}
            }
        }
        if !types.is_empty() {
            filter.type_ = Some(Cow::Owned(types));
        }
        filter
    }
}
