use leptos::prelude::*;
use mason_wheeler_shared::User;

#[derive(Clone, Debug)]
pub struct AuthState {
    pub user: ReadSignal<Option<User>>,
    pub loading: ReadSignal<bool>,
    pub refetch: Callback<()>,
}

pub fn provide_auth_state() -> AuthState {
    let (user, set_user) = signal::<Option<User>>(None);
    let (loading, set_loading) = signal(true);

    // Fetch current user on mount
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match crate::api::client::api_get::<User>("/api/auth/me").await {
                Ok(u) => {
                    set_user.set(Some(u));
                    set_loading.set(false);
                }
                Err(_) => {
                    set_user.set(None);
                    set_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        set_loading.set(false);
    }

    let refetch = Callback::new(move |()| {
        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async move {
                match crate::api::client::api_get::<User>("/api/auth/me").await {
                    Ok(u) => set_user.set(Some(u)),
                    Err(_) => set_user.set(None),
                }
            });
        }
    });

    let state = AuthState { user, loading, refetch };
    leptos::context::provide_context(state.clone());
    state
}

pub fn use_auth() -> AuthState {
    leptos::context::use_context::<AuthState>()
        .expect("use_auth() called outside of a context that provides AuthState")
}
