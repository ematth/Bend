(CC.lang program) =
  let callcc  = λcallback (λ$garbage($hole) (callback λ$hole(0)))
  let result  = (program callcc)
  let garbage = $garbage
  result

Main = (CC.lang λcallcc
  (+ 10 (callcc λk(+ (k 42) 1729))))
