pred = @n switch n {
  0: 0
  _: n-1
}

pred2 n = switch n {
  0: 0
  1: 0
  _: n-2
}

pred3 0 = 0
pred3 1 = 0
pred3 2 = 0
pred3 x = (- x 3)

zero 0 = 1
zero 1 = 0
zero * = 0

main = *
