<script>
	import { onMount } from 'svelte';

	// Data for demonstration
	let points = [
		{ x: -50, y: -50 },
		{ x: 0, y: 0 },
		{ x: 50, y: 50 }
		// Add more points as needed
	];

	let svg;

	onMount(() => {
		// Scale or transform the SVG as needed
		// This can be dynamic based on the points or user interaction
	});

	/**
	 * @param {{ x: any; y: any; }} point
	 */
	function transformPoint(point) {
		// Transform function to map your unbounded points to the SVG coordinate system
		// This is a placeholder function and should be replaced with your actual logic
		return {
			x: point.x + 200, // Offset for demonstration
			y: point.y + 200 // Offset for demonstration
		};
	}
</script>

<div class="graph">
	<svg bind:this={svg} width="400" height="400">
		{#each points as point}
			<circle cx={transformPoint(point).x} cy={transformPoint(point).y} r="3" fill="blue" />
		{/each}
		<!-- Add more SVG elements as needed for axes, labels, etc. -->
	</svg>
</div>

<style>
	.graph {
		width: 400px;
		height: 400px;
		border: 1px solid black;
	}
</style>
