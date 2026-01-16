// ============================================================================
// Chess Game Viewer - JavaScript Application
// ============================================================================

// Piece Unicode characters
const PIECES = {
    'K': '♔', 'Q': '♕', 'R': '♖', 'B': '♗', 'N': '♘', 'P': '♙',
    'k': '♚', 'q': '♛', 'r': '♜', 'b': '♝', 'n': '♞', 'p': '♟',
    'A': 'A', 'a': 'a'  // Amazon - use letter since no Unicode exists
};

// ============================================================================
// State Management
// ============================================================================

const state = {
    games: [],
    currentGameIndex: 0,
    currentMoveIndex: 0,
    board: null,  // 8x8 array
    initialFen: '8/8/4k3/4r3/8/8/8/3AK3 w - -',
    sideToMove: 'w',
    moveHistory: [],  // Array of {from, to, san}
    boardHistory: []  // Array of board states for navigation
};

// ============================================================================
// FEN Parsing & Board Representation
// ============================================================================

function parseFen(fen) {
    const board = Array(8).fill(null).map(() => Array(8).fill(null));
    const parts = fen.trim().split(/\s+/);
    const placement = parts[0];
    const sideToMove = parts[1] || 'w';
    
    const ranks = placement.split('/');
    for (let row = 0; row < 8 && row < ranks.length; row++) {
        let col = 0;
        for (const char of ranks[row]) {
            if (col >= 8) break;
            if (char >= '1' && char <= '8') {
                col += parseInt(char);
            } else {
                board[row][col] = char;
                col++;
            }
        }
    }
    
    return { board, sideToMove };
}

function boardToFen(board, sideToMove) {
    let fen = '';
    for (let row = 0; row < 8; row++) {
        let empty = 0;
        for (let col = 0; col < 8; col++) {
            const piece = board[row][col];
            if (piece) {
                if (empty > 0) {
                    fen += empty;
                    empty = 0;
                }
                fen += piece;
            } else {
                empty++;
            }
        }
        if (empty > 0) fen += empty;
        if (row < 7) fen += '/';
    }
    return `${fen} ${sideToMove} - -`;
}

function cloneBoard(board) {
    return board.map(row => [...row]);
}

// ============================================================================
// Move Parsing & Execution
// ============================================================================

function parseSquare(sq) {
    if (!sq || sq.length < 2) return null;
    const col = sq.charCodeAt(0) - 'a'.charCodeAt(0);
    const row = 8 - parseInt(sq[1]);
    if (col < 0 || col > 7 || row < 0 || row > 7) return null;
    return { row, col };
}

function squareToString(row, col) {
    return String.fromCharCode('a'.charCodeAt(0) + col) + (8 - row);
}

function findPiece(board, pieceChar, targetRow, targetCol, fromFile, fromRank) {
    // Find a piece that can move to the target square
    const isWhite = pieceChar === pieceChar.toUpperCase();
    const searchPiece = pieceChar.toUpperCase();
    
    const candidates = [];
    
    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const piece = board[row][col];
            if (!piece) continue;
            
            const pieceIsWhite = piece === piece.toUpperCase();
            if (pieceIsWhite !== isWhite) continue;
            
            const pieceType = piece.toUpperCase();
            if (pieceType !== searchPiece) continue;
            
            // Check file/rank disambiguation
            if (fromFile !== null && col !== fromFile) continue;
            if (fromRank !== null && row !== fromRank) continue;
            
            // Check if piece can reach target
            if (canPieceMove(pieceType, row, col, targetRow, targetCol, board)) {
                candidates.push({ row, col });
            }
        }
    }
    
    return candidates.length === 1 ? candidates[0] : candidates[0]; // Return first match
}

function canPieceMove(pieceType, fromRow, fromCol, toRow, toCol, board) {
    const dr = toRow - fromRow;
    const dc = toCol - fromCol;
    
    switch (pieceType) {
        case 'K':
            return Math.abs(dr) <= 1 && Math.abs(dc) <= 1;
        
        case 'R':
            if (dr !== 0 && dc !== 0) return false;
            return isPathClear(fromRow, fromCol, toRow, toCol, board);
        
        case 'Q':
            if (dr !== 0 && dc !== 0 && Math.abs(dr) !== Math.abs(dc)) return false;
            return isPathClear(fromRow, fromCol, toRow, toCol, board);
        
        case 'B':
            if (Math.abs(dr) !== Math.abs(dc)) return false;
            return isPathClear(fromRow, fromCol, toRow, toCol, board);
        
        case 'N':
            return (Math.abs(dr) === 2 && Math.abs(dc) === 1) ||
                   (Math.abs(dr) === 1 && Math.abs(dc) === 2);
        
        case 'A': // Amazon = Queen + Knight
            // Knight move
            if ((Math.abs(dr) === 2 && Math.abs(dc) === 1) ||
                (Math.abs(dr) === 1 && Math.abs(dc) === 2)) {
                return true;
            }
            // Queen move
            if (dr === 0 || dc === 0 || Math.abs(dr) === Math.abs(dc)) {
                return isPathClear(fromRow, fromCol, toRow, toCol, board);
            }
            return false;
        
        case 'P':
            // Simplified pawn logic
            const direction = board[fromRow][fromCol] === 'P' ? -1 : 1;
            if (dc === 0 && dr === direction && !board[toRow][toCol]) return true;
            if (dc === 0 && dr === 2 * direction && !board[toRow][toCol] && 
                ((direction === -1 && fromRow === 6) || (direction === 1 && fromRow === 1))) return true;
            if (Math.abs(dc) === 1 && dr === direction && board[toRow][toCol]) return true;
            return false;
        
        default:
            return false;
    }
}

function isPathClear(fromRow, fromCol, toRow, toCol, board) {
    const dr = Math.sign(toRow - fromRow);
    const dc = Math.sign(toCol - fromCol);
    
    let r = fromRow + dr;
    let c = fromCol + dc;
    
    while (r !== toRow || c !== toCol) {
        if (board[r][c]) return false;
        r += dr;
        c += dc;
    }
    return true;
}

function applyMove(board, san, isWhite) {
    // Parse SAN (Standard Algebraic Notation)
    let move = san.replace(/[+#!?]+$/, ''); // Remove check/mate/annotation symbols
    
    // Handle castling
    if (move === 'O-O' || move === 'O-O-O') {
        // Not implemented for this variant
        return null;
    }
    
    // Capture indicator
    const isCapture = move.includes('x');
    move = move.replace('x', '');
    
    // Promotion (not common in this variant)
    let promotion = null;
    if (move.includes('=')) {
        promotion = move.split('=')[1][0];
        move = move.split('=')[0];
    }
    
    // Determine piece type
    let pieceType = 'P';
    if (move[0] >= 'A' && move[0] <= 'Z') {
        pieceType = move[0];
        move = move.substring(1);
    }
    
    // Target square is last 2 characters
    const targetSquare = move.slice(-2);
    const target = parseSquare(targetSquare);
    if (!target) return null;
    
    // Disambiguation (file or rank before target)
    let fromFile = null;
    let fromRank = null;
    const disambig = move.slice(0, -2);
    for (const char of disambig) {
        if (char >= 'a' && char <= 'h') {
            fromFile = char.charCodeAt(0) - 'a'.charCodeAt(0);
        } else if (char >= '1' && char <= '8') {
            fromRank = 8 - parseInt(char);
        }
    }
    
    // Find the piece
    const pieceChar = isWhite ? pieceType : pieceType.toLowerCase();
    const from = findPiece(board, pieceChar, target.row, target.col, fromFile, fromRank);
    if (!from) return null;
    
    // Make the move
    const newBoard = cloneBoard(board);
    newBoard[target.row][target.col] = newBoard[from.row][from.col];
    newBoard[from.row][from.col] = null;
    
    // Handle promotion
    if (promotion) {
        newBoard[target.row][target.col] = isWhite ? promotion : promotion.toLowerCase();
    }
    
    return {
        board: newBoard,
        from: { row: from.row, col: from.col },
        to: { row: target.row, col: target.col }
    };
}

// ============================================================================
// PGN Parsing
// ============================================================================

function parsePgn(pgnText) {
    const games = [];
    
    // Split into individual games
    const gameTexts = pgnText.split(/\n\n(?=\[)/).filter(g => g.trim());
    
    for (const gameText of gameTexts) {
        const game = {
            headers: {},
            moves: [],
            fen: null,
            result: '*'
        };
        
        // Parse headers
        const headerRegex = /\[(\w+)\s+"([^"]*)"\]/g;
        let match;
        while ((match = headerRegex.exec(gameText)) !== null) {
            game.headers[match[1]] = match[2];
        }
        
        // Get FEN if present
        game.fen = game.headers['FEN'] || null;
        game.result = game.headers['Result'] || '*';
        
        // Parse moves - everything after headers
        const moveSection = gameText.replace(/\[[^\]]*\]/g, '').trim();
        
        // Extract individual moves (handles move numbers and results)
        const moveRegex = /([KQRBNAP]?[a-h]?[1-8]?x?[a-h][1-8](?:=[QRBN])?[+#]?|O-O-O|O-O)/g;
        let moveMatch;
        while ((moveMatch = moveRegex.exec(moveSection)) !== null) {
            game.moves.push(moveMatch[1]);
        }
        
        if (game.moves.length > 0 || game.fen) {
            games.push(game);
        }
    }
    
    return games;
}

// ============================================================================
// Board Rendering
// ============================================================================

function createBoard() {
    const boardElement = document.getElementById('board');
    boardElement.innerHTML = '';
    
    for (let row = 0; row < 8; row++) {
        for (let col = 0; col < 8; col++) {
            const square = document.createElement('div');
            square.className = `square ${(row + col) % 2 === 0 ? 'light' : 'dark'}`;
            square.dataset.row = row;
            square.dataset.col = col;
            boardElement.appendChild(square);
        }
    }
}

function renderBoard(board, lastMove = null) {
    const boardElement = document.getElementById('board');
    const squares = boardElement.querySelectorAll('.square');
    
    squares.forEach((square, index) => {
        const row = Math.floor(index / 8);
        const col = index % 8;
        const piece = board[row][col];
        
        // Reset classes
        square.className = `square ${(row + col) % 2 === 0 ? 'light' : 'dark'}`;
        
        // Highlight last move
        if (lastMove) {
            if ((row === lastMove.from.row && col === lastMove.from.col) ||
                (row === lastMove.to.row && col === lastMove.to.col)) {
                square.classList.add('last-move');
            }
        }
        
        // Render piece
        if (piece) {
            const isWhite = piece === piece.toUpperCase();
            const pieceUpper = piece.toUpperCase();
            
            if (pieceUpper === 'A') {
                // Amazon - special rendering
                square.innerHTML = `<span class="amazon piece-${isWhite ? 'white' : 'black'}">${isWhite ? 'A' : 'a'}</span>`;
            } else {
                const symbol = PIECES[piece] || piece;
                square.innerHTML = `<span class="piece-${isWhite ? 'white' : 'black'}">${symbol}</span>`;
            }
        } else {
            square.innerHTML = '';
        }
    });
    
    // Update turn indicator
    updateTurnIndicator();
}

function updateTurnIndicator() {
    const turnColor = document.getElementById('turn-color');
    if (state.sideToMove === 'w') {
        turnColor.textContent = 'White';
        turnColor.className = 'turn-color white';
    } else {
        turnColor.textContent = 'Black';
        turnColor.className = 'turn-color black';
    }
}

// ============================================================================
// Game Navigation
// ============================================================================

function loadGame(index) {
    if (index < 0 || index >= state.games.length) return;
    
    state.currentGameIndex = index;
    const game = state.games[index];
    
    // Parse initial position
    const fen = game.fen || state.initialFen;
    const parsed = parseFen(fen);
    state.board = parsed.board;
    state.sideToMove = parsed.sideToMove;
    
    // Build board history
    state.boardHistory = [{ board: cloneBoard(state.board), sideToMove: state.sideToMove, lastMove: null }];
    state.moveHistory = [];
    
    let currentBoard = cloneBoard(state.board);
    let currentSide = state.sideToMove;
    
    for (const san of game.moves) {
        const isWhite = currentSide === 'w';
        const result = applyMove(currentBoard, san, isWhite);
        
        if (result) {
            currentBoard = result.board;
            currentSide = isWhite ? 'b' : 'w';
            state.moveHistory.push({ san, from: result.from, to: result.to });
            state.boardHistory.push({
                board: cloneBoard(currentBoard),
                sideToMove: currentSide,
                lastMove: { from: result.from, to: result.to }
            });
        }
    }
    
    // Go to initial position
    state.currentMoveIndex = 0;
    goToMove(0);
    
    // Update move list
    renderMoveList();
    
    // Update game info
    updateGameInfo(game);
    
    // Show result
    showGameResult(game.result);
}

function goToMove(index) {
    if (index < 0) index = 0;
    if (index >= state.boardHistory.length) index = state.boardHistory.length - 1;
    
    state.currentMoveIndex = index;
    const historyEntry = state.boardHistory[index];
    
    state.board = cloneBoard(historyEntry.board);
    state.sideToMove = historyEntry.sideToMove;
    
    renderBoard(state.board, historyEntry.lastMove);
    updateMoveCounter();
    highlightCurrentMove();
    updateNavigationButtons();
}

function updateMoveCounter() {
    const counter = document.getElementById('move-counter');
    const total = state.boardHistory.length - 1;
    counter.textContent = `${state.currentMoveIndex} / ${total}`;
}

function updateNavigationButtons() {
    const total = state.boardHistory.length - 1;
    document.getElementById('btn-first').disabled = state.currentMoveIndex === 0;
    document.getElementById('btn-prev').disabled = state.currentMoveIndex === 0;
    document.getElementById('btn-next').disabled = state.currentMoveIndex >= total;
    document.getElementById('btn-last').disabled = state.currentMoveIndex >= total;
}

function renderMoveList() {
    const moveList = document.getElementById('move-list');
    
    if (state.moveHistory.length === 0) {
        moveList.innerHTML = '<p class="no-moves">No moves in this game</p>';
        return;
    }
    
    let html = '';
    for (let i = 0; i < state.moveHistory.length; i++) {
        const move = state.moveHistory[i];
        const moveNum = Math.floor(i / 2) + 1;
        
        if (i % 2 === 0) {
            html += `<span class="move-number">${moveNum}.</span>`;
        }
        
        html += `<span class="move" data-index="${i + 1}">${move.san}</span> `;
    }
    
    moveList.innerHTML = html;
    
    // Add click handlers
    moveList.querySelectorAll('.move').forEach(el => {
        el.addEventListener('click', () => {
            goToMove(parseInt(el.dataset.index));
        });
    });
}

function highlightCurrentMove() {
    const moveList = document.getElementById('move-list');
    moveList.querySelectorAll('.move').forEach(el => {
        el.classList.remove('current');
        if (parseInt(el.dataset.index) === state.currentMoveIndex) {
            el.classList.add('current');
            el.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
    });
}

function updateGameInfo(game) {
    const info = document.getElementById('game-info');
    const white = game.headers['White'] || 'White';
    const black = game.headers['Black'] || 'Black';
    info.textContent = `${white} vs ${black}`;
}

function showGameResult(result) {
    const resultDiv = document.getElementById('game-result');
    const resultText = document.getElementById('result-text');
    
    resultDiv.classList.remove('hidden', 'white-wins', 'black-wins', 'draw');
    
    if (result === '1-0') {
        resultDiv.classList.add('white-wins');
        resultText.textContent = '1-0 White wins!';
    } else if (result === '0-1') {
        resultDiv.classList.add('black-wins');
        resultText.textContent = '0-1 Black wins!';
    } else if (result === '1/2-1/2') {
        resultDiv.classList.add('draw');
        resultText.textContent = '½-½ Draw';
    } else {
        resultDiv.classList.add('hidden');
    }
}

function updateGameSelector() {
    const select = document.getElementById('game-select');
    select.innerHTML = '';
    
    if (state.games.length === 0) {
        select.innerHTML = '<option value="0">No games loaded</option>';
        return;
    }
    
    state.games.forEach((game, index) => {
        const white = game.headers['White'] || 'White';
        const black = game.headers['Black'] || 'Black';
        const result = game.result || '*';
        const option = document.createElement('option');
        option.value = index;
        option.textContent = `Game ${index + 1}: ${white} vs ${black} (${result})`;
        select.appendChild(option);
    });
}

// ============================================================================
// Event Handlers
// ============================================================================

function setupEventListeners() {
    // Navigation buttons
    document.getElementById('btn-first').addEventListener('click', () => goToMove(0));
    document.getElementById('btn-prev').addEventListener('click', () => goToMove(state.currentMoveIndex - 1));
    document.getElementById('btn-next').addEventListener('click', () => goToMove(state.currentMoveIndex + 1));
    document.getElementById('btn-last').addEventListener('click', () => goToMove(state.boardHistory.length - 1));
    
    // Keyboard navigation
    document.addEventListener('keydown', (e) => {
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
        
        switch (e.key) {
            case 'ArrowLeft':
                goToMove(state.currentMoveIndex - 1);
                break;
            case 'ArrowRight':
                goToMove(state.currentMoveIndex + 1);
                break;
            case 'Home':
                goToMove(0);
                break;
            case 'End':
                goToMove(state.boardHistory.length - 1);
                break;
        }
    });
    
    // Game selector
    document.getElementById('game-select').addEventListener('change', (e) => {
        loadGame(parseInt(e.target.value));
    });
    
    // Load PGN button
    document.getElementById('btn-load-pgn').addEventListener('click', () => {
        document.getElementById('pgn-file-input').click();
    });
    
    // File input
    document.getElementById('pgn-file-input').addEventListener('change', (e) => {
        const file = e.target.files[0];
        if (file) {
            const reader = new FileReader();
            reader.onload = (event) => {
                loadPgnText(event.target.result);
            };
            reader.readAsText(file);
        }
    });
    
    // Parse PGN from textarea
    document.getElementById('btn-parse-pgn').addEventListener('click', () => {
        const text = document.getElementById('pgn-text').value;
        if (text.trim()) {
            loadPgnText(text);
        }
    });
    
    // FEN editor
    document.getElementById('btn-edit-fen').addEventListener('click', () => {
        const editor = document.getElementById('fen-editor');
        const fenInput = document.getElementById('fen-input');
        editor.classList.toggle('hidden');
        if (!editor.classList.contains('hidden')) {
            fenInput.value = boardToFen(state.board, state.sideToMove);
            fenInput.focus();
        }
    });
    
    document.getElementById('btn-apply-fen').addEventListener('click', () => {
        const fen = document.getElementById('fen-input').value.trim();
        if (fen) {
            applyCustomFen(fen);
        }
    });
    
    document.getElementById('btn-cancel-fen').addEventListener('click', () => {
        document.getElementById('fen-editor').classList.add('hidden');
    });
}

function loadPgnText(text) {
    state.games = parsePgn(text);
    updateGameSelector();
    
    if (state.games.length > 0) {
        loadGame(0);
    }
}

function applyCustomFen(fen) {
    try {
        const parsed = parseFen(fen);
        state.board = parsed.board;
        state.sideToMove = parsed.sideToMove;
        state.initialFen = fen;
        
        // Create a single-state game
        state.boardHistory = [{ board: cloneBoard(state.board), sideToMove: state.sideToMove, lastMove: null }];
        state.moveHistory = [];
        state.currentMoveIndex = 0;
        
        renderBoard(state.board);
        updateMoveCounter();
        renderMoveList();
        updateNavigationButtons();
        
        document.getElementById('fen-editor').classList.add('hidden');
        document.getElementById('game-result').classList.add('hidden');
    } catch (e) {
        alert('Invalid FEN string');
    }
}

// ============================================================================
// Initialization
// ============================================================================

function init() {
    createBoard();
    
    // Load default position
    const parsed = parseFen(state.initialFen);
    state.board = parsed.board;
    state.sideToMove = parsed.sideToMove;
    state.boardHistory = [{ board: cloneBoard(state.board), sideToMove: state.sideToMove, lastMove: null }];
    
    renderBoard(state.board);
    updateMoveCounter();
    updateNavigationButtons();
    
    setupEventListeners();
    
    // Try to load games.pgn automatically
    fetch('games.pgn')
        .then(response => response.text())
        .then(text => {
            if (text && text.includes('[')) {
                loadPgnText(text);
            }
        })
        .catch(() => {
            // No games.pgn found, that's okay
        });
}

// Start the app
document.addEventListener('DOMContentLoaded', init);
