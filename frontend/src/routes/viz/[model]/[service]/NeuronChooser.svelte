<script lang="ts">
	import { goto } from '$app/navigation';
	import { VIZ_EXT } from '$lib/base';
	import type { ModelMetadata } from '$lib/modelMetadata';
	import { get } from 'svelte/store';
	import {
		navLayerIndexStore,
		navLayerIndices,
		navNeuronIndexStore,
		navNeuronIndices
	} from './stores';

	export let modelMetadata: ModelMetadata;
	export let serviceName: string;

	$: ({ name: modelName, numLayers, layerSize } = modelMetadata);

	let navLayerIndex = 0;
	let navNeuronIndex = 0;
	$: navLayerIndex = get(navLayerIndices)[modelName] ?? 0;
	$: navNeuronIndex = get(navNeuronIndices)[modelName] ?? 0;

	function updateIndices(layerIndex: number, neuronIndex: number) {
		navLayerIndices.update((indices) => ({ ...indices, [modelName]: layerIndex }));
		navNeuronIndices.update((indices) => ({ ...indices, [modelName]: neuronIndex }));

		navLayerIndexStore.set(layerIndex);
		navNeuronIndexStore.set(neuronIndex);
	}

	$: updateIndices(navLayerIndex, navNeuronIndex);

	let navigating: boolean = false;

	function goToNeuron() {
		navigating = true;
		const url = `/${VIZ_EXT}/${modelName}/${serviceName}/${navLayerIndex}/${navNeuronIndex}`;
		goto(url);
	}

	function goToRandom() {
		navigating = true;
		const layerIndex = Math.floor(Math.random() * numLayers);
		const neuronIndex = Math.floor(Math.random() * layerSize);
		const url = `/${VIZ_EXT}/${modelName}/${serviceName}/${layerIndex}/${neuronIndex}`;
		goto(url);
	}
</script>

<form on:submit|preventDefault={goToNeuron}>
	<table>
		<tr><th>Model</th><th>Layer</th><th>Neuron</th></tr>
		<tr
			><td>{modelMetadata.name}</td><td
				><input
					name="layer-index"
					type="number"
					bind:value={navLayerIndex}
					min="0"
					max={numLayers - 1}
					placeholder="Layer index..."
				/></td
			><td
				><input
					name="neuron-index"
					type="number"
					bind:value={navNeuronIndex}
					min="0"
					max={layerSize - 1}
					placeholder="Neuron index..."
				/></td
			><td
				><button>Go!</button>
				<button on:click|preventDefault={goToRandom}>Random</button>
				{#if navigating}Loading...{/if}</td
			></tr
		>
	</table>
</form>
