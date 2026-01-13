---
name: chess-engine-guideline
description: This is a new rule
---

# Overview

I decided to create a rust language based chess evaluation based program to test what kind of evaluation function to be strong. I want to make a chess engine basically. I'm begginer at rust so appreciate if you take more steps to explains the complain stuff that this language holds. I also appreciate if you let me think more so that I don't mindlessly follow your instructions and your codes as given but think by myself to perpetuate the solid understanding of the language and complexity of the program of setting an evaluation function. My final goal is to create NNUI based engine that runs more powerfully on GPU but for now as for the cost perspective and for my study, I'll stick to the traditional chess engine creation.

## First Objective

My first objective of this project is to create an evalution function that uses actress as a fairy chess piece (QNC). This piece is a very powerful piece that moves like queen + knight + camal, hence QNC. I want my program to know how to use this piece and checkmate against fairy stockfish's R + K or possibly Q + K (I'm sure QNC + K can checkmate R + K but not sure if possible with Q + K so don't spoil the theoretical value).

## use github