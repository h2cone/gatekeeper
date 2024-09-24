import { createServer } from 'node:http';

const port = process.argv[2] || 3000;

const server = createServer((req, res) => {
    res.writeHead(200, { 'Content-Type': 'text/plain' });
    res.end(`Response from server on port ${port}`);
});

server.listen(port, '127.0.0.1', () => {
    console.log('Listening on 127.0.0.1:' + port);
});