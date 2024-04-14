use crate::{
    error_template::{AppError, ErrorTemplate},
    short::ShortenUrl,
};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Html lang="en" dir="ltr"/>

        <Meta charset="utf-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1"/>

        <Stylesheet id="leptos" href="/pkg/shorty.css"/>
        <Link rel="icon" type_="image/x-icon" href="favicon.ico"/>

        <Title text="Short(y) URL"/>

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let shorten_url = create_server_action::<ShortenUrl>();
    let value = shorten_url.value();
    let fetching = shorten_url.pending();

    let is_err = move || value.with(|v| matches!(v, Some(Err(_))));

    let short_url = move || {
        value.with(|v| {
            if let Some(resp) = v {
                resp.clone().map(|short| short.to_string())
            } else {
                Ok(String::new())
            }
        })
    };

    view! {
        <h1>Url Shortener</h1>

        <ActionForm class="url-form" action=shorten_url>
            <input
                type="url"
                name="url"
                placeholder="Your url..."
                class:error=is_err
            />
        </ActionForm>

        <Spinner fetching=fetching/>

        <ErrorBoundary fallback=|errors| {
            view! {
                <p id="errors">
                    Error:
                    {move || {
                        errors.get().into_iter().map(|(_, e)| e.to_string()).collect_view()
                    }}

                </p>
            }
        }>
            <Show when=move || !fetching()>
                <code>{short_url}</code>
            </Show>
        </ErrorBoundary>
    }
}

#[component]
pub fn Spinner(#[prop(into)] fetching: Signal<bool>) -> impl IntoView {
    view! {
        <Show when=fetching>
            <svg
                class="spinner"
                xmlns="http://www.w3.org/2000/svg"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <line x1="12" x2="12" y1="2" y2="6"></line>
                <line x1="12" x2="12" y1="18" y2="22"></line>
                <line x1="4.93" x2="7.76" y1="4.93" y2="7.76"></line>
                <line x1="16.24" x2="19.07" y1="16.24" y2="19.07"></line>
                <line x1="2" x2="6" y1="12" y2="12"></line>
                <line x1="18" x2="22" y1="12" y2="12"></line>
                <line x1="4.93" x2="7.76" y1="19.07" y2="16.24"></line>
                <line x1="16.24" x2="19.07" y1="7.76" y2="4.93"></line>
            </svg>
        </Show>
    }
}
