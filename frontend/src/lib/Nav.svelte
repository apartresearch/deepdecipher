<script>
	import Title from './Title.svelte';

	let showMenu = false;
</script>

<div class="navbar">
	<div class="title"><Title /></div>
	<div class="nav-links">
		<a href="http://127.0.0.1:5173/doc" target="_blank">Docs</a>
		<a href="https://apartresearch.com/r/deepdecipher" target="_blank">About</a>
		<a href="https://github.com/apartresearch/deepdecipher" target="_blank">Github</a>
	</div>
</div>

<style>
	.navbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		max-width: 1140px;
		margin: 0 auto;
		padding: 0 20px;
		padding-bottom: 5px;
	}

	.nav-links {
		display: flex;
		gap: 20px;
	}

	.nav-links > a {
		color: #333;
		font-weight: 400;
		text-decoration: none;
	}

	.nav-links > a:hover {
		text-decoration: underline;
	}

	@media (max-width: 600px) {
		.navbar {
			flex-direction: column;
			align-items: center;
			gap: 10px;
		}
	}
</style>
