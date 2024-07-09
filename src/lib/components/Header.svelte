<script>
    let links = [
        { href: '/', text: 'Home' },
        { href: '/essays', text: 'Essays' },
        { href: '/bio', text: 'Bio' }
    ];
</script>

<header class="bg-gray-900 p-4">
    <nav>
        <ul class="flex space-x-4">
            {#each links as link}
                <li><a href={link.href} class="text-white hover:text-gray-100">{link.text}</a></li>
            {/each}
        </ul>
    </nav>
</header>