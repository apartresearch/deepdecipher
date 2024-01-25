import { env } from '$env/dynamic/public';

//export const BASE_URL: string = "https://deepdecipher.org"
export const BASE_API_ORIGIN: string = env.PUBLIC_BACKEND_ORIGIN || "http://127.0.0.1"
export const BASE_API_PORT: number = parseInt(env.PUBLIC_BACKEND_PORT || "8080")
if (isNaN(BASE_API_PORT)) {
    throw new Error(`Invalid backend port given in environment varible \`BACKEND_PORT\` '${BASE_API_PORT}' is not a number.`)
}
if (BASE_API_PORT < 0 || BASE_API_PORT > 65535) {
    throw new Error(`Invalid backend port given in environment varible \`BACKEND_PORT\` '${BASE_API_PORT}' is not a valid port number.`)
}
export const BASE_API_URL: string = `${BASE_API_ORIGIN}:${BASE_API_PORT}`
export const VIZ_EXT: string = "viz";
export const API_EXT: string = "api";

export function formatNumber(num: number, maxDecimals: number): string {
    return num.toFixed(maxDecimals).replace(/\.?0+$/, "");
}
