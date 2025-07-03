import init, { WasmGomoku, board_size } from '../pkg/gomoku.js';

let BOARD_SIZE;

const canvas = document.getElementById('board');
const startButton = document.getElementById('startButton');
const messageDiv = document.getElementById('message');

const gl = canvas.getContext('webgl');
if (!gl) {
    alert('WebGL not supported');
}

let game;
let gameOver = false;

function endGame(msg) {
    messageDiv.textContent = msg;
    gameOver = true;
    startButton.disabled = false;
    startButton.textContent = 'Restart';
}

// Basic shaders for 2D rendering
const vertCode = `
attribute vec2 coordinates;
void main(void) {
    gl_Position = vec4(coordinates, 0.0, 1.0);
}`;

const fragCode = `
precision mediump float;
uniform vec4 uColor;
void main(void) {
    gl_FragColor = uColor;
}`;

const vertShader = gl.createShader(gl.VERTEX_SHADER);
gl.shaderSource(vertShader, vertCode);
gl.compileShader(vertShader);

const fragShader = gl.createShader(gl.FRAGMENT_SHADER);
gl.shaderSource(fragShader, fragCode);
gl.compileShader(fragShader);

const shaderProgram = gl.createProgram();
gl.attachShader(shaderProgram, vertShader);
gl.attachShader(shaderProgram, fragShader);
gl.linkProgram(shaderProgram);

const coord = gl.getAttribLocation(shaderProgram, 'coordinates');
const colorUniform = gl.getUniformLocation(shaderProgram, 'uColor');

gl.useProgram(shaderProgram);
gl.enableVertexAttribArray(coord);

const BOARD_SCALE = 0.95; // keep some margin around the board so stones at
                         // the edge aren't clipped

function ndcFromBoard(row, col) {
    const x = -BOARD_SCALE + (col / (BOARD_SIZE - 1)) * 2 * BOARD_SCALE;
    const y = BOARD_SCALE - (row / (BOARD_SIZE - 1)) * 2 * BOARD_SCALE;
    return [x, y];
}

function drawGrid() {
    const vertices = [];
    for (let i = 0; i < BOARD_SIZE; i++) {
        const t1 = ndcFromBoard(i, 0);
        const t2 = ndcFromBoard(i, BOARD_SIZE - 1);
        vertices.push(t1[0], t1[1], t2[0], t2[1]);

        const s1 = ndcFromBoard(0, i);
        const s2 = ndcFromBoard(BOARD_SIZE - 1, i);
        vertices.push(s1[0], s1[1], s2[0], s2[1]);
    }
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(vertices), gl.STATIC_DRAW);
    gl.vertexAttribPointer(coord, 2, gl.FLOAT, false, 0, 0);
    gl.uniform4f(colorUniform, 0.0, 0.0, 0.0, 1.0);
    gl.drawArrays(gl.LINES, 0, vertices.length / 2);
}

function circleVertices(x, y, radius) {
    const segments = 40;
    const verts = [];
    verts.push(x, y);
    for (let i = 0; i <= segments; i++) {
        const angle = (i / segments) * Math.PI * 2;
        verts.push(x + radius * Math.cos(angle));
        verts.push(y + radius * Math.sin(angle));
    }
    return verts;
}

function drawStone(row, col, player) {
    const [x, y] = ndcFromBoard(row, col);
    const verts = circleVertices(x, y, (2 * BOARD_SCALE / BOARD_SIZE) * 0.4);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(verts), gl.STATIC_DRAW);
    gl.vertexAttribPointer(coord, 2, gl.FLOAT, false, 0, 0);
    if (player === 1) {
        gl.uniform4f(colorUniform, 0.0, 0.0, 0.0, 1.0);
    } else {
        gl.uniform4f(colorUniform, 1.0, 1.0, 1.0, 1.0);
    }
    gl.drawArrays(gl.TRIANGLE_FAN, 0, verts.length / 2);
}

function boardMatrix() {
    const data = game.board();
    const board = [];
    for (let r = 0; r < BOARD_SIZE; r++) {
        board[r] = [];
        for (let c = 0; c < BOARD_SIZE; c++) {
            board[r][c] = data[r * BOARD_SIZE + c];
        }
    }
    return board;
}

function render() {
    gl.clearColor(0.95, 0.92, 0.8, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    drawGrid();
    const b = boardMatrix();
    for (let r = 0; r < BOARD_SIZE; r++) {
        for (let c = 0; c < BOARD_SIZE; c++) {
            if (b[r][c] !== 0) {
                drawStone(r, c, b[r][c]);
            }
        }
    }
}

function startGame() {
    game = new WasmGomoku();
    gameOver = false;
    messageDiv.textContent = '';
    startButton.disabled = true; // disable startButton when game is started.
    render();
}

canvas.addEventListener('click', (e) => {
    if (gameOver || !game) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const col = Math.floor(x / (canvas.width / BOARD_SIZE));
    const row = Math.floor(y / (canvas.height / BOARD_SIZE));
    if (!game.make_move(row, col)) return;
    render();
    let winner = game.check_winner();
    if (winner === 1 || game.is_board_full()) { // combine this 'if' and following 'if' into one. because both contents of 'if' are similiar.
        endGame(winner === 1 ? 'You win!' : 'Draw!');
        return;
    }
    game.switch_player();
    const aiMove = game.ai_move();
    game.make_move(aiMove[0], aiMove[1]);
    render();
    winner = game.check_winner();
    if (winner === 2 || game.is_board_full()) { // combine this 'if' and following 'if' into one. because both contents of 'if' are similiar.
        endGame(winner === 2 ? 'AI wins' : 'Draw!');
        return;
    }
    game.switch_player();
});

startButton.addEventListener('click', startGame);

init().then(() => {
    BOARD_SIZE = board_size();
    game = new WasmGomoku();
    render();
});
