import init, { WasmGomoku, board_size } from '../pkg/gomoku.js';

let BOARD_SIZE;

const canvas = document.getElementById('board');
const startButton = document.getElementById('startButton');
const messageDiv = document.getElementById('message');
const infoDiv = document.getElementById('info');
const playerFirstRadio = document.getElementById('playerFirst');
const aiFirstRadio = document.getElementById('aiFirst');

const gl = canvas.getContext('webgl');
if (!gl) {
    alert('WebGL not supported');
}

let game;
let gameOver = false;
let recentMoves = [];
let animRequestId = null;
let lastMove = null;
let currentBoard = [];

const FADE_DURATION = 1000; // ms
const HIGHLIGHT_DURATION = 2000; // ms

// Display a message and reset state when the game finishes.
function endGame(msg) {
    messageDiv.textContent = msg;
    gameOver = true;
    startButton.disabled = false;
    startButton.textContent = 'Restart';
    playerFirstRadio.disabled = false;
    aiFirstRadio.disabled = false;
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

// Convert board coordinates to normalized device coordinates used by WebGL.
function ndcFromBoard(row, col) {
    const x = -BOARD_SCALE + (col / (BOARD_SIZE - 1)) * 2 * BOARD_SCALE;
    const y = BOARD_SCALE - (row / (BOARD_SIZE - 1)) * 2 * BOARD_SCALE;
    return [x, y];
}

// Render the board grid lines.
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

// Generate vertices for a circle centered at (x, y) with the given radius.
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

// Draw a stone for the specified player at board position (row, col).
// `alpha` controls transparency for fade-in animations.
function drawStone(row, col, player, alpha = 1.0) {
    const [x, y] = ndcFromBoard(row, col);
    const verts = circleVertices(x, y, (2 * BOARD_SCALE / BOARD_SIZE) * 0.4);
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(verts), gl.STATIC_DRAW);
    gl.vertexAttribPointer(coord, 2, gl.FLOAT, false, 0, 0);
    if (player === 1) {
        gl.uniform4f(colorUniform, 0.0, 0.0, 0.0, alpha);
    } else {
        gl.uniform4f(colorUniform, 1.0, 1.0, 1.0, alpha);
    }
    gl.drawArrays(gl.TRIANGLE_FAN, 0, verts.length / 2);
}

// Outline the most recent move with a pulsing highlight.
function drawHighlight(row, col, player, alpha) {
    const [x, y] = ndcFromBoard(row, col);
    const radius = (2 * BOARD_SCALE / BOARD_SIZE) * 0.48;
    const segments = 40;
    const verts = [];
    for (let i = 0; i <= segments; i++) {
        const angle = (i / segments) * Math.PI * 2;
        verts.push(x + radius * Math.cos(angle));
        verts.push(y + radius * Math.sin(angle));
    }
    const buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(verts), gl.STATIC_DRAW);
    gl.vertexAttribPointer(coord, 2, gl.FLOAT, false, 0, 0);
    if (player === 1) {
        gl.uniform4f(colorUniform, 0.0, 0.0, 0.0, alpha);
    } else {
        gl.uniform4f(colorUniform, 1.0, 1.0, 1.0, alpha);
    }
    gl.drawArrays(gl.LINE_STRIP, 0, verts.length / 2);
}


// Convert the flat board array from WebAssembly into a 2D matrix.
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

// Draw the current board state and animate recent moves.
function render() {
    gl.clearColor(0.95, 0.92, 0.8, 1.0);
    gl.clear(gl.COLOR_BUFFER_BIT);
    drawGrid();
    const b = boardMatrix();
    currentBoard = b;
    const now = performance.now();
    let needAnim = false;
    const newRecent = [];
    for (let r = 0; r < BOARD_SIZE; r++) {
        for (let c = 0; c < BOARD_SIZE; c++) {
            const cell = b[r][c];
            if (cell === 0) continue;
            const anim = recentMoves.find(m => m.row === r && m.col === c);
            if (anim) {
                const elapsed = now - anim.time;
                const alpha = Math.min(elapsed / FADE_DURATION, 1.0);
                if (elapsed < FADE_DURATION) {
                    needAnim = true;
                    newRecent.push(anim);
                }
                drawStone(r, c, cell, alpha);
            } else {
                drawStone(r, c, cell, 1.0);
            }
        }
    }

    if (lastMove) {
        const elapsed = now - lastMove.time;
        if (elapsed < HIGHLIGHT_DURATION) {
            const alpha = 0.5 + 0.5 * Math.sin((elapsed / 150) * Math.PI);
            drawHighlight(lastMove.row, lastMove.col, lastMove.player, alpha);
            needAnim = true;
        } else {
            lastMove = null;
        }
    }
    recentMoves = newRecent;
    if (needAnim) {
        animRequestId = requestAnimationFrame(render);
    } else if (animRequestId) {
        cancelAnimationFrame(animRequestId);
        animRequestId = null;
    }
}

// Initialise a new game and optionally let the AI play first.
function startGame() {
    game = new WasmGomoku();
    gameOver = false;
    messageDiv.textContent = '';
    infoDiv.textContent = '';
    startButton.disabled = true; // disable startButton when game is started.
    playerFirstRadio.disabled = true;
    aiFirstRadio.disabled = true;
    recentMoves = [];
    lastMove = null;
    if (animRequestId) {
        cancelAnimationFrame(animRequestId);
        animRequestId = null;
    }
    render();
    if (aiFirstRadio.checked) {
        const aiMove = game.ai_move();
        game.make_move(aiMove[0], aiMove[1]);
        const aiNow = performance.now();
        recentMoves.push({ row: aiMove[0], col: aiMove[1], player: 2, time: aiNow });
        lastMove = { row: aiMove[0], col: aiMove[1], player: 2, time: aiNow };
        render();
        const winner = game.check_winner();
        if (winner === 2 || game.is_board_full()) {
            endGame(winner === 2 ? 'AI wins' : 'Draw!');
            return;
        }
        game.switch_player();
    }
}

canvas.addEventListener('click', (e) => {
    if (gameOver || !game) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const col = Math.floor(x / (canvas.width / BOARD_SIZE));
    const row = Math.floor(y / (canvas.height / BOARD_SIZE));
    if (!game.make_move(row, col)) return;
    const now = performance.now();
    recentMoves.push({ row, col, player: 1, time: now });
    lastMove = { row, col, player: 1, time: now };
    render();
    let winner = game.check_winner();
    if (winner === 1 || game.is_board_full()) { // combine this 'if' and following 'if' into one. because both contents of 'if' are similiar.
        endGame(winner === 1 ? 'You win!' : 'Draw!');
        return;
    }
    game.switch_player();
    const aiMove = game.ai_move();
    game.make_move(aiMove[0], aiMove[1]);
    const aiNow = performance.now();
    recentMoves.push({ row: aiMove[0], col: aiMove[1], player: 2, time: aiNow });
    lastMove = { row: aiMove[0], col: aiMove[1], player: 2, time: aiNow };
    render();
    winner = game.check_winner();
    if (winner === 2 || game.is_board_full()) { // combine this 'if' and following 'if' into one. because both contents of 'if' are similiar.
        endGame(winner === 2 ? 'AI wins' : 'Draw!');
        return;
    }
    game.switch_player();
});

canvas.addEventListener('mousemove', (e) => {
    if (!game) return;
    const rect = canvas.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;
    const col = Math.floor(x / (canvas.width / BOARD_SIZE));
    const row = Math.floor(y / (canvas.height / BOARD_SIZE));
    if (row < 0 || row >= BOARD_SIZE || col < 0 || col >= BOARD_SIZE) {
        infoDiv.textContent = '';
        return;
    }
    const val = game.evaluation_at(row, col);
    if (val === undefined || currentBoard[row][col] !== 0) {
        infoDiv.textContent = `(${row}, ${col}): N/A`;
    } else {
        infoDiv.textContent = `(${row}, ${col}): ${val}`;
    }
});

startButton.addEventListener('click', startGame);

init().then(() => {
    BOARD_SIZE = board_size();
    game = new WasmGomoku();
    render();
});
