<script>
    export let title;
    export let date;
    export let slug;
    export let excerpt;
</script>

<article class="mb-8">
    <h2 class="text-l font-bold mb-2"><a href="/essays/{slug}" class="text-gray-900 hover:text-gray-950">{title}</a></h2>
    <p class="text-gray-900 italic mb-2">{date}</p>
    <p>{excerpt}</p>
</article>