import aiohttp
import asyncio
import time
import random
import sys

# Number of requests you want to send concurrently per batch
CONCURRENT_REQUESTS = 10
# Number of batches you'd like to run
BATCHES = 10

LOG_PATH = sys.argv[1] if len(sys.argv) > 1 else None


def url() -> str:
    neuron = random.randint(0, 2047)
    return f"https://deepdecipher.org/viz/solu-1l/all/0/{neuron}"
    # return f"https://neuroscope.io/solu-1l/0/{neuron}.html"
    # return f"https://openaipublic.blob.core.windows.net/neuron-explainer/data/explanations/0/{neuron}.jsonl"
    # return f"https://openaipublic.blob.core.windows.net/neuron-explainer/neuron-viewer/index.html#/layers/0/neurons/{neuron}"


async def fetch(session) -> any:
    request_url = url()
    try:
        async with session.get(request_url) as response:
            data = await response.read()

            if response.status >= 400:
                if LOG_PATH:
                    with open(LOG_PATH, "a") as f:
                        f.write(f"{request_url}\n")
                        f.write(f"{response}\n")
                return False, data
            else:
                return True, data
    except aiohttp.ClientError:
        return False, None


async def send_batch(session) -> (int, int, int):
    tasks = []
    for _ in range(CONCURRENT_REQUESTS):
        task = asyncio.ensure_future(fetch(session))
        tasks.append(task)
    results = await asyncio.gather(*tasks)
    total_data = sum([len(data) for _, data in results if data is not None])
    successful_requests = sum([successful for successful, _ in results])
    return total_data, successful_requests


async def main() -> (int, int, int):
    async with aiohttp.ClientSession() as session:
        total_data = 0
        successful_requests = 0
        for _ in range(BATCHES):
            (
                total_batch_data,
                batch_successful_requests,
            ) = await send_batch(session)
            total_data += total_batch_data
            successful_requests += batch_successful_requests
    return total_data, successful_requests


if __name__ == "__main__":
    start_time = time.time()

    loop = asyncio.get_event_loop()
    total_data, successful_requests = loop.run_until_complete(main())
    loop.close()

    end_time = time.time()

    total_requests = CONCURRENT_REQUESTS * BATCHES
    total_time = end_time - start_time
    rps = total_requests / total_time

    # Convert total data from bytes to MB
    total_data_mb = total_data / (1024 * 1024)
    data_per_second = total_data_mb / total_time

    print(f"Sent {total_requests} requests in {total_time:.2f} seconds")
    print(f"Average RPS: {rps:.2f}")
    print(f"Transferred {total_data_mb:.2f} MB in total")
    print(f"Average data transferred per second: {data_per_second:.2f} MB/s")
    print(f"Successful requests: {successful_requests}")
    print(f"Failed requests: {total_requests - successful_requests}")
