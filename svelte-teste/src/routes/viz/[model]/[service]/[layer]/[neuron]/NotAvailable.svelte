<script lang="ts">
	export let message: string;
</script>

<div class="not_available">
	{message} is not available for this neuron.
</div>

<style>
	.not_available {
		background-color: rgba(255, 0, 0, 0.1);
		border: 1px dashed rgba(255, 0, 0, 0.5);
		color: rgba(255, 0, 0, 0.8);
		border-radius: 0.5em;
		padding: 0.4em 1em;
		display: inline-block;
	}
</style>
