# halo2_bubble_sort
### Fangzhou Yu

## Goal
Given an unsorted array as input, use bubble sort algorithm to sort the array within the circuit implemented using halo2.

## Assumptions
We know the length of array beforehand.

## Gate: questionable
For unchanged, we check each one x0-x1=0\
For swapped, say x and y, we check x0 - x1(y0) + y0 - y1(x0) = 0

## Note
anything other than main is unsuccessful attempt