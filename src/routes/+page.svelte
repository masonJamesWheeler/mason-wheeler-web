<script>
    import EssayPreview from '$lib/components/EssayPreview.svelte';
    import { essays } from '$lib/essays.js';
</script>

<svelte:head>
    <title>Mason Wheeler</title>
</svelte:head>

<section>
    <h2 class="text-xl font-semibold mb-4">Latest Essays</h2>
    {#each essays.slice(0, 3) as essay}
        <EssayPreview {...essay} />
    {/each}
    <a href="/essays" class="text-blue-600 hover:text-blue-800">View all essays</a>
</section>