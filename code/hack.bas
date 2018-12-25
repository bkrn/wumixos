I            REM  +------------------------------------------------+
II           REM  | HACK.BAS      (c) 19100   fr33 v4r14bl3z       |
III          REM  |                                                |
IV           REM  | Brute-forces passwords on UM vIX.0 systems.    |
V            REM  | Compile with Qvickbasic VII.0 or later:        |
VI           REM  |    /bin/qbasic hack.bas                        |
VII          REM  | Then run:                                      |
VIII         REM  |   ./hack.exe username                          |
IX           REM  |                                                |
X            REM  | This program is for educational purposes only! |
XI           REM  +------------------------------------------------+
XII          REM
XIII         IF ARGS() > I THEN GOTO XIX
XIV          PRINT "usage: ./hack.exe username"
XV           PRINT CHR(X)
XVI          END
XVII         REM
XVIII        REM  get username from command line
XIX          DIM username AS STRING
XX           username = ARG(II)
XXI          REM  common words used in passwords
XXII         DIM pwdcount AS INTEGER
XXIII        pwdcount = LIII
XXIV         DIM words(pwdcount) AS STRING
XXV          words(I) = "airplane"
XXVI         words(II) = "alphabet"
XXVII        words(III) = "aviator"
XXVIII       words(IV) = "bidirectional"
XXIX         words(V) = "changeme"
XXX          words(VI) = "creosote"
XXXI         words(VII) = "cyclone"
XXXII        words(VIII) = "december"
XXXIII       words(IX) = "dolphin"
XXXIV        words(X) = "elephant"
XXXV         words(XI) = "ersatz"
XXXVI        words(XII) = "falderal"
XXXVII       words(XIII) = "functional"
XXXVIII      words(XIV) = "future"
XXXIX        words(XV) = "guitar"
XL           words(XVI) = "gymnast"
XLI          words(XVII) = "hello"
XLII         words(XVIII) = "imbroglio"
XLIII        words(XIX) = "january"
XLIV         words(XX) = "joshua"
XLV          words(XXI) = "kernel"
XLVI         words(XXII) = "kingfish"
XLVII        words(XXIII) = "(\b.bb)(\v.vv)"
XLVIII       words(XXIV) = "millennium"
XLIX         words(XXV) = "monday"
L            words(XXVI) = "nemesis"
LI           words(XXVII) = "oatmeal"
LII          words(XXVIII) = "october"
LIII         words(XXIX) = "paladin"
LIV          words(XXX) = "pass"
LV           words(XXXI) = "password"
LVI          words(XXXII) = "penguin"
LVII         words(XXXIII) = "polynomial"
LVIII        words(XXXIV) = "popcorn"
LIX          words(XXXV) = "qwerty"
LX           words(XXXVI) = "sailor"
LXI          words(XXXVII) = "swordfish"
LXII         words(XXXVIII) = "symmetry"
LXIII        words(XXXIX) = "system"
LXIV         words(XL) = "tattoo"
LXV          words(XLI) = "thursday"
LXVI         words(XLII) = "tinman"
LXVII        words(XLIII) = "topography"
LXVIII       words(XLIV) = "unicorn"
LXIX         words(XLV) = "vader"
LXX          words(XLVI) = "vampire"
LXXI         words(XLVII) = "viper"
LXXII        words(XLVIII) = "warez"
LXXIII       words(XLIX) = "xanadu"
LXXIV        words(L) = "xyzzy"
LXXV         words(LI) = "zephyr"
LXXVI        words(LII) = "zeppelin"
LXXVII       words(LIII) = "zxcvbnm"
LXXVIII      REM try each password
LXXIX        PRINT "attempting hack with " + pwdcount + " passwords " + CHR(X)
LXXX         DIM i AS INTEGER
LXXXI        i = I
LXXXII       IF CHECKPASS(username, words(i)) THEN GOTO LXXXVI
LXXXIII      i = i + I
LXXXIV       IF i > pwdcount THEN GOTO LXXXIX
LXXXV        GOTO LXXXII
LXXXVI       PRINT "found match!! for user " + username + CHR(X)
LXXXVII      PRINT "password: " + words(i) + CHR(X)
LXXXVIII     END
LXXXIX       PRINT "no simple matches for user " + username + CHR(X)
XC           REM
XCI          REM  the above code will probably crack passwords for many
XCII         REM  users so I always try it first. when it fails, I try the
XCIII        REM  more expensive method below.
XCIV         REM
XCV          REM  passwords often take the form
XCVI         REM    dictwordDD
XCVII        REM  where DD is a two-digit decimal number. try these next:
XCVIII       i = I
XCIX         DIM ones AS INTEGER
C            DIM tens AS INTEGER
CI           ones = XLVIII
CII          tens = XLVIII
CIII         IF CHECKPASS(username, words(i)+CHR(ones)+CHR(tens)) THEN GOTO CXIV
CIV          REM PRINT words(i) + CHR(ones) + CHR(tens) + CHR(X)
CV           ones = ones + I
CVI          IF ones < LVIII THEN GOTO CXIII
CVII         ones = XLVIII
CVIII        tens = tens + I
CIX          IF tens < LVIII THEN GOTO CXIII
CX           tens = XLVIII
CXI          i = i + I
CXII         IF i > pwdcount THEN GOTO CXVII
CXIII        GOTO CIII
CXIV         PRINT "found match!! for user " + username + CHR(X)
CXV          PRINT "password: " + words(i) + CHR(ones) + CHR(tens) + CHR(X)
CXVI         END
CXVII        PRINT "no complex matches for user " + username + CHR(X)
CXVIII       END
