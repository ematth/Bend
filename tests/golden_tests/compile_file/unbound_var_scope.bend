# Fully parenthesized, this is (λa ((λb b) b)).
# Since applications must have (), the second 'b' is not in scope
main = λa (λb b b)
