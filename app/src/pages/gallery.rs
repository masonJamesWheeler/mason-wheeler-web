use leptos::prelude::*;

// (filename, alt text, category)
const PHOTOS: &[(&str, &str, &str)] = &[
    ("exterior-front-walkway-sunny.jpg", "Front of home, sunny day", "Exterior"),
    ("exterior-front-street-view-sunny.jpg", "Street view with mature tree", "Exterior"),
    ("exterior-front-close-up.jpg", "Front facade close-up", "Exterior"),
    ("exterior-side-yard-corner-lot.jpg", "Side yard, corner lot", "Exterior"),
    ("exterior-side-lawn-landscaping.jpg", "Side lawn with landscaping", "Exterior"),
    ("living-room-fireplace-wide.jpg", "Living room with stone fireplace", "Living Room"),
    ("living-room-fireplace-angle-2.jpg", "Living room, alternate angle", "Living Room"),
    ("living-room-fireplace-closeup.jpg", "Stone fireplace detail", "Living Room"),
    ("living-room-windows-seating.jpg", "Living room seating area", "Living Room"),
    ("living-room-entry-kitchen-view.jpg", "Living room toward kitchen", "Living Room"),
    ("living-room-front-door-entry.jpg", "Entry and front door", "Living Room"),
    ("kitchen-dining-nook-wide.jpg", "Kitchen with dining nook", "Kitchen"),
    ("kitchen-range-hood-appliances.jpg", "Kitchen appliances and range hood", "Kitchen"),
    ("kitchen-gas-range-subway-tile.jpg", "Gas range with subway tile", "Kitchen"),
    ("bedroom-1-primary-dark-furniture.jpg", "Primary bedroom", "Bedrooms"),
    ("bedroom-2-queen-green-accent.jpg", "Second bedroom, green accent wall", "Bedrooms"),
    ("den-wide-shelves-tv.jpg", "Den with shelving and TV area", "Den"),
    ("den-sliding-doors-stairs.jpg", "Den with sliding doors to yard", "Den"),
    ("den-sliding-doors-backyard-view.jpg", "Den, backyard view", "Den"),
    ("backyard-firepit-shed-fence.jpg", "Backyard fire pit and shed", "Outdoor"),
    ("backyard-sliding-doors-patio.jpg", "Patio with chairs", "Outdoor"),
    ("aerial-drone-rear-yard.jpg", "Aerial view, rear of property", "Aerial"),
    ("aerial-drone-overhead-lot-outline.jpg", "Overhead lot view", "Aerial"),
];

#[component]
pub fn GalleryPage() -> impl IntoView {
    let (selected, set_selected) = signal::<Option<usize>>(None);

    let photo_count = PHOTOS.len();

    view! {
        <div class="max-w-6xl mx-auto px-6 py-10">
            <div class="mb-8">
                <a href="/" class="text-sm text-stone-400 hover:text-stone-600 transition-colors">"← Back"</a>
                <h1 class="text-2xl font-semibold text-stone-900 tracking-tight mt-3">"Photo Gallery"</h1>
                <p class="text-stone-500 text-sm mt-1">{format!("{} photos · 8404 12th Ave S, Seattle", photo_count)}</p>
            </div>

            // Photo grid
            <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2">
                {PHOTOS.iter().enumerate().map(|(i, (file, alt, _cat))| {
                    let file = file.to_string();
                    let alt = alt.to_string();
                    view! {
                        <button
                            class="relative aspect-[4/3] overflow-hidden rounded-xl group cursor-pointer focus:outline-none focus:ring-2 focus:ring-amber-500 focus:ring-offset-2"
                            on:click=move |_| set_selected.set(Some(i))
                        >
                            <img
                                src={format!("/photos/{}", file)}
                                alt={alt}
                                class="absolute inset-0 w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                loading="lazy"
                            />
                            <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors duration-200" />
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>

        // Lightbox
        <Show when=move || selected.get().is_some()>
            <div
                class="fixed inset-0 z-[100] bg-black/95 flex items-center justify-center"
                on:click=move |_| set_selected.set(None)
            >
                // Close button
                <button
                    class="absolute top-4 right-4 z-10 w-10 h-10 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 text-white transition-colors"
                    on:click=move |_| set_selected.set(None)
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>

                // Previous
                <button
                    class="absolute left-4 z-10 w-10 h-10 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 text-white transition-colors"
                    on:click=move |e| {
                        e.stop_propagation();
                        set_selected.update(|s| {
                            if let Some(idx) = s {
                                if *idx > 0 { *idx -= 1; }
                            }
                        });
                    }
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
                    </svg>
                </button>

                // Next
                <button
                    class="absolute right-4 z-10 w-10 h-10 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 text-white transition-colors"
                    on:click=move |e| {
                        e.stop_propagation();
                        set_selected.update(|s| {
                            if let Some(idx) = s {
                                if *idx < photo_count - 1 { *idx += 1; }
                            }
                        });
                    }
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                    </svg>
                </button>

                // Image
                {move || {
                    selected.get().map(|idx| {
                        let (file, alt, _cat) = PHOTOS[idx];
                        view! {
                            <div class="flex flex-col items-center max-w-5xl max-h-[85vh] px-16" on:click=move |e| e.stop_propagation()>
                                <img
                                    src={format!("/photos/{}", file)}
                                    alt={alt.to_string()}
                                    class="max-h-[75vh] max-w-full object-contain rounded-lg"
                                />
                                <p class="mt-3 text-white/40 text-xs">{format!("{} of {}", idx + 1, photo_count)}</p>
                            </div>
                        }
                    })
                }}
            </div>
        </Show>
    }
}
