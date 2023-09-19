//export const BASE_URL: string = "https://deepdecipher.org"
export const BASE_API_URL: string = "http://127.0.0.1:8080"
export const VIZ_EXT: string = "viz";
export const API_EXT: string = "api";

export function formatNumber(num: number, maxDecimals: number): string {
    return num.toFixed(maxDecimals).replace(/\.?0+$/, "");
}
