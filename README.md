# halo2_bubble_sort
### Fangzhou Yu

## Goal
Given an unsorted array as input, use bubble sort algorithm to sort the array within the circuit implemented using halo2.

## Assumptions
Array contains only non-negative entires.\
We know the length of array beforehand.\
The maxium difference between smallest and largest element in the array is less than 100

## My Solution
Let the array A to have length of k. In the table, each row contains k + 1 + 2 columns. Representing difference between two\
elements (discussed below) and two selectors. \
In the first row, load the unsorted array and make the diff <- A_k - A_k \
After ith round of sorting, we know that the last i elements of A must be sorted, so the ith item from the end is the new element\
to be sorted at this round. So we compare A_i+1 and A_i to see whether the ith element is sorted correctly. To do so, we do a range\
check on the diff column and check it is between 0 and 100. This way we have made a zkp of the array is sorted.

## Current problems
Implementation issues: 
- chip and config, where to put what
- implementation of bubble sort
 - how to access the ith and (i+1)th element of array