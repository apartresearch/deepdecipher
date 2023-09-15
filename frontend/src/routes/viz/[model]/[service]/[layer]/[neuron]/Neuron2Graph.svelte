<script lang="ts">
	import { instance } from '@viz-js/viz';

	export let graphString: string;

	const renderGraph = async (graphString: string) => {
		return await instance().then((viz) => {
			const output = viz.render(graphString, { format: 'svg', engine: 'dot' });
			if (output.output === undefined) {
				throw new Error(`Graph couldn't be rendered. Errors: ${output.errors}`);
			}
			return output.output;
		});
	};

	$: renderGraphFuture = renderGraph(graphString);
</script>

{#await renderGraphFuture}
	<div class="neuron2graph">Rendering graph...</div>
{:then graph}
	<div class="neuron2graph">{@html graph}</div>
{:catch error}
	<div class="neuron2graph">{error.message}</div>
{/await}
