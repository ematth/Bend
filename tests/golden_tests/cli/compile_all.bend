type Pair
  = (Pair fst snd)

Pair.get f (Pair/Pair fst snd) = (f fst snd)

main = (Pair.get @x @y (+ x y) (Pair/Pair 40 2))
