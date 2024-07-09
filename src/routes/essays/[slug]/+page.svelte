<script>
    import { page } from '$app/stores';
    import { essays } from '$lib/essays.js';

    $: essay = essays.find(e => e.slug === $page.params.slug);
</script>

<svelte:head>
    <title>{essay.title} | Mason Wheeler</title>
</svelte:head>

<article class="max-w-3xl mx-auto">
    <h1 class="text-xl font-bold mb-2">{essay.title}</h1>
    <p class="text-gray-600 italic mb-6">{essay.date}</p>
    <div class="prose prose-sm max-w-none">
        {@html essay.content}
    </div>
</article>

<style>
    :global(.prose h2) {
        @apply text-xl font-semibold mt-6 mb-3;
    }
    :global(.prose h3) {
        @apply text-lg font-medium mt-4 mb-2;
    }
    :global(.prose p) {
        @apply mb-3;
    }
    :global(.prose ul) {
        @apply list-disc pl-5 mb-3 space-y-1;
    }
</style>