<script>
	// set the type
	/**
	 * @type {any}
	 */
	export let title;
	/**
	 * @type {any}
	 */
	export let description;
</script>

<div class="hero">
	<div class="content">
		<h2 class="title">{title}</h2>
		<p class="description">{description}</p>
	</div>
</div>

<style>
	.hero {
		display: flex;
		max-width: 500px;
		background: linear-gradient(135deg, #6e8efb, #a777e3);
		color: #ffffff;
		padding: 20px 25px;
		box-shadow: 4px 4px 6px rgba(13, 59, 90, 0.05);
		border-radius: 8px;
	}

	.title {
		margin: 0;
		margin-bottom: 0.25em;
	}

	.description {
		margin: 0;
		opacity: 0.9; /* Slight transparency for blending */
		line-height: 1.6; /* Improved line spacing for readability */
	}
</style>
