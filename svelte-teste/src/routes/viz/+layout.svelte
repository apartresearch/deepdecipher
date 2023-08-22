<slot />

<style>
	:global(body) {
		font-family: 'IBM Plex Sans', sans-serif;
	}

	:global(h1) {
		margin: 0.5em 0;
	}
	:global(h2) {
		white-space: pre-wrap;
	}
	:global(.container) {
		max-width: 60em;
		margin: auto 0;
	}

	:global(.meta-data) {
		padding: 0 1em;
		border-left: 1px solid black;
	}

	:global(.meta-data.first) {
		border-left: none;
		padding-left: 0;
	}

	:global(.code) {
		margin: 0;
		padding: 0.1em 0;
		line-height: 1.6em;
		color: rgba(0, 0, 0, 0.8);
		background-color: rgba(0, 0, 0, 0.05);
		white-space: pre-wrap;
		border: 1px solid rgba(34, 0, 0, 0.1);
		font-family: 'IBM Plex Mono', monospace;
	}

	:global(.not-available) {
		background-color: rgba(255, 0, 0, 0.1);
		border: 1px dashed rgba(255, 0, 0, 0.5);
		color: rgba(255, 0, 0, 0.8);
		border-radius: 0.5em;
		padding: 0.4em 1em;
		display: inline-block;
	}
</style>
