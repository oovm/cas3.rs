(set (attrs True) (list locked protected))
(set (attrs False) (list locked protected))

(set (and True True) True)
(set (and True False) False)
(set (and False True) False)
(set (and False False) False)

(set (or True True) True)
(set (or True False) True)
(set (or False True) True)
(set (or False False) False)

(set (not True) False)
(set (not False) True)

(set (not (not (pattern x (blank)))) x)
