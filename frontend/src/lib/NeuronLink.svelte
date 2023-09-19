<script lang="ts">
	import { VIZ_EXT } from '$lib/base';

	export let modelName: String;
	export let layer: number;
	export let neuron: number;
</script>

<a class="result" href="/{VIZ_EXT}/{modelName}/all/{layer}/{neuron}" target="_blank"
	>{layer}:{neuron} ↗</a
>

<style>
	.result {
		min-width: 9em;
		padding: 0.5em 0.2em;
		text-decoration: none;
		/*color: black;*/
	}

	.result:hover {
		color: rgba(0, 0, 0, 0.7);
	}
</style>
