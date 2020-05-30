use crate::{sec::OAuth2, serde::Serialize, types::Id};
use std::borrow::Cow;

/// Basic Data container required for the base template
#[derive(Debug, Clone, Serialize)]
pub struct BaseView<'a> {
    /// The modification time of the js file
    pub js_mtime: u64,
    /// The modification time of the js header file
    pub jsh_mtime: u64,
    /// The modification time of the css file
    pub style_mtime: u64,
    /// The title of the application
    pub title: Cow<'a, str>,
    /// The version of the application
    pub version: Option<&'a str>,
}

/// Basic Container required for template rendering
#[derive(Debug, Clone, Serialize)]
pub struct BaseContainer<'a, T: Serialize, P: Serialize> {
    /// An optional csrf token. Required if the view requires one
    pub csrf: Option<String>,
    /// Relativ url which points to the app root
    pub base: Cow<'a, str>,
    /// BaseView containing values for the base template
    pub main: &'a BaseView<'a>,
    /// The notifications to display to the user
    pub noti: Option<&'a [Notification<'a>]>,
    /// Parameters used when calling the site
    pub param: &'a P,
    /// The current user
    pub user: UserContainer<'a>,
    /// The subtype required for each template
    pub sub: &'a T,
    /// The current path
    pub path: &'a str,
}

impl<'a, T: Serialize, P: Serialize> BaseContainer<'a, T, P> {
    /// Creates a new `BaseContainer` with as many default values as possible
    #[inline]
    pub fn new(
        view: &'a BaseView<'a>,
        sub: &'a T,
        param: &'a P,
        path: &'a str,
    ) -> Self {
        Self {
            csrf: None,
            base: Cow::Borrowed("./"),
            main: view,
            noti: None,
            param,
            user: UserContainer::default(),
            sub,
            path,
        }
    }
}

/// Basic Container containing information about the current user
#[derive(Debug, Clone, Serialize)]
pub struct UserContainer<'a> {
    /// UUID of the User
    pub id: Option<&'a Id>,
    /// The uid of the user. `None` if no user is logged in
    pub uid: Option<&'a str>,
    /// The name of the user. `None` if no user is logged in or no name is set
    pub name: Option<&'a str>,
    /// Whether the current user is an admin
    pub is_admin: bool,
    /// Whether the current user is a superuser
    pub is_superuser: bool,
}

impl Default for UserContainer<'_> {
    #[inline]
    fn default() -> Self {
        Self {
            id: None,
            uid: None,
            name: None,
            is_admin: false,
            is_superuser: false,
        }
    }
}

/// A notification which is displayed to the user.
/// Contains more information whether the current operation worked or not
#[derive(Debug, Clone, Serialize)]
pub enum Notification<'a> {
    /// Display an information about a successful deletion
    Deleted {
        /// Name of the Type
        name: &'static str,
    },
    /// Display an error with a like to the help page
    Error {
        /// Name of the Type
        name: &'static str,
        /// Name of the parameter
        para: &'static str,
        /// Link to the help page
        help: &'static str,
    },
    /// Display an information linking to the new type
    Info {
        /// Name of the Type
        name: &'static str,
        /// Base Linke
        url: &'static str,
        /// Id of the new object
        id: Cow<'a, Id>,
    },
    /// Display an error showing that a similar type already exists
    Unique {
        /// Name of the Type
        name: &'static str,
        /// Name of the parameter
        para: &'static str,
        /// Link to the help page
        help: &'static str,
    },
}

/// Basic Data required for the application
#[derive(Debug, Clone)]
pub struct BaseData {
    /// App Secret used for encryption and signing inside the application
    pub app_secret: [u8; 32],
    /// `OAuth2` client
    pub oauth: OAuth2,
}
