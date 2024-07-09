<script>
    import EssayPreview from '$lib/components/EssayPreview.svelte';
    import { essays } from '$lib/essays.js';
</script>

<svelte:head>
    <title>Essays | Mason Wheeler</title>
</svelte:head>

<h1 class="text-xl font-bold mb-8">Essays</h1>

{#each essays as essay}
    <EssayPreview {...essay} />
{/each}