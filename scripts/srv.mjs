import { createServer } from 'node:http';

const server = createServer((req, res) => {
    if (req.url === '/json') {
        res.setHeader('Content-Type', 'application/json');
        const response = { message: 'Hello, World!' };
        res.end(JSON.stringify(response) + '\n');
    } else {
        res.statusCode = 404;
        res.end('Not Found\n');
    }
});

server.listen(3000, () => {
    console.log('Listening on :3000');
});