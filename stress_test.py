import aiohttp
import asyncio
import time
import random

# Number of requests you want to send concurrently per batch
CONCURRENT_REQUESTS = 10
# Number of batches you'd like to run
BATCHES = 10

total_data = 0  # This will store the total data transferred in bytes
failed_requests = 0  # Counter for failed requests
successful_requests = 0


def url() -> str:
    neuron = random.randint(0, 2047)
    # return f"https://deepdecipher.org/api/solu-1l/all/0/{neuron}"
    # return f"https://neuroscope.io/solu-1l/0/{neuron}.html"
    return f"https://openaipublic.blob.core.windows.net/neuron-explainer/data/explanations/0/{neuron}.jsonl"
    # return f"https://openaipublic.blob.core.windows.net/neuron-explainer/neuron-viewer/index.html#/layers/0/neurons/{neuron}"


async def fetch(session):
    global total_data
    global failed_requests
    global successful_requests
    request_url = url()
    try:
        async with session.get(request_url) as response:
            if response.status >= 400:
                failed_requests += 1
            else:
                successful_requests += 1

            data = await response.read()
            total_data += len(data)
            return data
    except aiohttp.ClientError:
        failed_requests += 1
        return b""


async def send_batch(session):
    tasks = []
    for _ in range(CONCURRENT_REQUESTS):
        task = asyncio.ensure_future(fetch(session))
        tasks.append(task)

    await asyncio.gather(*tasks)


async def main():
    async with aiohttp.ClientSession() as session:
        for _ in range(BATCHES):
            await send_batch(session)


if __name__ == "__main__":
    start_time = time.time()

    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())

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
    print(f"Failed requests: {failed_requests}")
