# Board Representation 

## 1. Mailbox vs Bitboards vs 0x88

### Mailbox

- The term, Mailbox comes from the resemblance of each square of the board holding its data as an individual memory according to CPW (Chess Programming Wiki).

>The square number, or its file and rank, acts like an address to a post box, which might be empty or may contain one chess piece.

#### Pros

- Simplicity and easy maintainability
- Good for beginners since it is straightforward

#### Cons

- May not be the best option if your ultimate goal is the efficiency of the calculation
    - Compared with bitboards, it uses many loop and branch commands such as for, while, if


### Bitboards

#### Pros

- holds the information of position as bits so it is faster to calculate (without loop)
  - For example, for the calculation of legal rook movements or attacks you can use the precalculated mask table and magic bitboards

**Visual image:**
```
Mailbox:
┌─┬─┬─┬─┬─┬─┬─┬─┐
│?│?│?│?│?│?│?│?│  ← 1マスずつチェック
├─┼─┼─┼─┼─┼─┼─┼─┤     (64回の if + メモリアクセス)
│?│?│?│?│?│?│?│?│
├─┼─┼─┼─┼─┼─┼─┼─┤
│?│?│?│?│?│?│?│?│
...

Bitboards:
1101001011010010110100101101001011010010110100101101001011010010
↑ 64ビット整数1つで全マスを表現
↑ 1回の演算で全マスを処理
```

#### Cons

- harder to understand what's going on for the beginner

### 0x88

- Alternative option for Mailbox and Bitboards but exist in Array-based board representation just as Mailbox. Bitboards belongs to Bitmap-based BP.

- 0x88 uses additional value outside of the board to make the calculation easier. 10x12 Board uses the same idea but less additional value called Sentinel.

#### Pros

- Array-based BP so it is easy to understand for human but optimized to calculate faster

#### Cons

- Could be hard to understand some concepts still...

## Conclusion

### Mailbox is the winner

- Reason: For the sake of this project, I wouldn't focus much about the efficiency of the calculation but more emphasis on the intuitive understanding of what is going on under the surface. For that purpose, mailbox is fitting because it is easy to understand and straightforward. I would like to use bitboards in the future.

## References

- https://www.chessprogramming.org/Mailbox
- https://www.chessprogramming.org/Bitboards
- https://www.chessprogramming.org/0x88
- https://www.chessprogramming.org/10x12_Board