<script lang="ts">
	export let token: string;
	export let activation: number;
	export let color: string;
</script>

<span class="token" style="background-color: {color}" data-tooltipe="{token}{'\n'}{activation}">
	{token}
</span>

<style>
	.token {
		margin: 0;
		padding: 0.2em 0;
		line-height: 1.6em;
		color: rgba(0, 0, 0, 0.8);
		white-space: pre-wrap;
		border: 1px solid rgba(34, 0, 0, 0.1);
	}
	.token:hover {
		filter: brightness(1.5);
	}
</style>
