; The roman numerals and their integer equivalent
(define numerals (list
    (cons "I" 1)
    (cons "V" 5)
    (cons "X" 10)
    (cons "L" 50)
    (cons "C" 100)
    (cons "D" 500)
    (cons "M" 1000)))


; Return the list of roman numerals that can be used
; to describe an integer `value` meaning the numeral
; greater than value and two lower than it
(define (get-wrapping-numerals value)
    (define (inner value before last candidates) 
            (if (> (cdar candidates) value)
                (list before last (car candidates))
                (inner value last (car candidates) (cdr candidates))))
    (inner value () (car numerals) (cdr numerals)))


(define (within value lower upper) (>= value (- (cdr upper) (cdr lower))))


; a non-null numeral can be used as a subtractor poistion in roman
; numerals if it is I X or C
(define (subtractor? numeral) (and (not (null? numeral)) 
                                        (member (car numeral) (list "I" "X" "C"))))


; get the roman numerals used to describe the largest (left most)
; part of the numeral translation of integer `value` 
; this will be one numeral unless subtractive
; notataion is used so C for 101 but IV for XC for 91
(define (get-numerals value) 
        (let* ((wrappers (get-wrapping-numerals value))
              (before (car wrappers)) 
              (last (cadr wrappers)) 
              (next (caddr wrappers)))
        (cond ((equal? (cdr last) value) (cons last ()))
              ((and (subtractor? before) (within value before next)) (list before next))
              ((and (subtractor? last) (within value last next)) (list last next))
              (else (cons last ())))))


; remove the value described by the numeral list `numerals`
; from the integer `initial` and return the new value
(define (value-step initial numerals) 
        (- initial (fold - (cdar numerals) (map cdr (cdr numerals)))))


(define (chars-step chars numerals) 
        (append chars (map car numerals)))


; generate a list of roman numerals describing `value` starting with the
; numerals in the list `chars`
; each iteration creates the left most remaining numeral
(define (roman-step value chars) 
    (let ((numerals (get-numerals value)))
    (if (equal? 0 value) 
        chars 
        (roman-step (value-step value numerals) (chars-step chars numerals)))))

; convert an integer to a roman numeral string
(define (as-roman value) (apply string-append (roman-step value ())))

(define (spaces count) (display " ")(if (not (equal? count 0)) (spaces (- count 1))))

(define (write-roman-line count line) (display (as-roman count))
                                      (spaces (- 12 (string-length (as-roman count))))
                                      (display line)
                                      (display "\n")
                                      (make-roman (+ count 1) (read-line)))

(define (make-roman count line) (if (not (eof-object? line)) (write-roman-line count line)))

; Append the roman numeral line number to every line in the path in-file and write
; the resulting lines to the path out-file
(define (romanize in-file out-file)
        (with-output-to-file out-file
        (lambda () (with-input-from-file in-file 
        (lambda () (make-roman 1 (read-line)))))))