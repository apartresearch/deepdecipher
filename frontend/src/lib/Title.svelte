<script>
	import CardDescription from './CardDescription.svelte';
</script>

<div class="logo">
	<img src="/logo.svg" alt="DeepDecipher logo" class="logo-img" />
	<h1><a href="/">DeepDecipher<sup>ⓘ</sup></a></h1>
	<div class="onhover">
		<CardDescription
			title="3 methods and 25 models"
			description="The website provides data on 25 models. All models have highest activating samples as scraped from neuroscope.io, 20 models have Neuron2Graph data and 2 models (gpt2-small and gpt2-xl) have GPT-4 explanations. This will almost certainly be expanded later."
		/>
	</div>
</div>

<style>
	sup {
		font-size: 0.5em;
		vertical-align: super;
		margin-left: 0.2em;
	}

	a {
		color: inherit;
		text-decoration: none;
	}

	.onhover {
		position: absolute;
		top: 55px;
		pointer-events: none;
		transition: opacity 0.1s;
		opacity: 0;
		z-index: 3;
	}

	.logo:hover .onhover {
		opacity: 0.95;
	}

	.logo {
		display: flex;
		align-items: center;
	}

	.logo-img {
		width: 40px;
		height: 40px;
		margin-right: 5px;
	}

	h1 {
		font-size: 1.5rem;
		margin: 0;
		font-weight: 400;
	}
</style>
