import cluster from 'cluster';
import os from 'os';

const numCPUs = os.cpus().length;

const numWorkersKey = 'NUM_WORKERS';
const numWorkers = numWorkersKey in process.env ? process.env[numWorkersKey] : "" + numCPUs;

if (cluster.isMaster) {
  for (let i = 0; i < numWorkers; i++) {
    cluster.fork();
  }

  cluster.on('exit', (worker) => {
    console.log(`Worker ${worker.process.pid} died`);
  });
} else {
  // Dynamically import server entry point
  import('./build/index.js').catch(err => console.error(err));
}

