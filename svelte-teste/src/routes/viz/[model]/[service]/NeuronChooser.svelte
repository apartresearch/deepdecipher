<script lang="ts">
	import { goto } from '$app/navigation';
	import { VIZ_EXT } from '$lib/base';
	import type { ModelMetadata } from '$lib/modelMetadata';

	export let modelMetadata: ModelMetadata;
	export let serviceName: string;

	$: ({ name, numLayers, layerSize } = modelMetadata);

	let navLayerIndex: number = 0;
	let navNeuronIndex: number = 0;

	$: targetUrl = `/${VIZ_EXT}/${name}/${serviceName}/${navLayerIndex}/${navNeuronIndex}`;
</script>

<form on:submit|preventDefault={() => goto(targetUrl)}>
	<table>
		<tr><th>Model</th><th>Layer</th><th>Neuron</th></tr>
		<tr
			><td>{modelMetadata.name}</td><td
				><input
					type="number"
					bind:value={navLayerIndex}
					min="0"
					max={numLayers - 1}
					placeholder="Layer index..."
				/></td
			><td
				><input
					type="number"
					bind:value={navNeuronIndex}
					min="0"
					max={layerSize - 1}
					placeholder="Neuron index..."
				/></td
			><td><button>Go!</button></td></tr
		>
	</table>
</form>
