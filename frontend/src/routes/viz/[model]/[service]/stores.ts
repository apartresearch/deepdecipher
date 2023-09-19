import { writable, type Writable } from 'svelte/store';

export const navLayerIndices: Writable<{ [modelName: string]: number; }> = writable({});
export const navNeuronIndices: Writable<{ [modelName: string]: number; }> = writable({});

export const navLayerIndexStore: Writable<number> = writable(0);
export const navNeuronIndexStore: Writable<number> = writable(0);