# Prime-Shootout: Final Benchmark Results

All benchmarks: 10 runs × 10 repeats per run
Platform: Windows 11, AMD/Intel x64
Compiler: GCC 13.2, -O3 -march=native -std=c++17

---

## Round 1: n = 500,000 (41,538 primes)

```
┌─────────────────────────┬──────────────┬─────────────┬──────┐
│ Implementation            				 │ Mean Avg (ms)	    │ Std Dev (ms)	   │ Runs       │
├─────────────────────────┼──────────────┼─────────────┼──────┤
│ c-primes-claude         				 │        24.90 	 		    │        4.05 			   │   10 	       │
│ c-primes-gpetey         				 │        25.20 	 		    │        6.65 			   │   10 	       │
│ c-primes-grok               				 │        25.57 	 		    │        3.99 			   │   10 	       │
│ c-primes-gemini         				 │        29.66 	 		    │       10.42   		   │   10 	       │
├─────────────────────────┼──────────────┼─────────────┼──────┤
│ r-primes-fast           				 │        31.45 			    │        5.91 			   │   10 	       │
│ r-primes                					 │        37.76 			    │       19.46 			   │   10 	       │
├─────────────────────────┼──────────────┼─────────────┼──────┤
│ c-primes-fast (baseline)		 │        37.96 			    │       23.51 			   │   10 	       │
│ c-primes (baseline)     			 │        40.55 			    │       17.10 			   │   10 	       │
├─────────────────────────┼──────────────┼─────────────┼──────┤
│ p-primes-smart-50k      			 │      2502.16 		    │      303.85 		   │   10 	       │
│ p-primes-smart-500      			 │      2524.25 		    │      287.17 		   │   10 	       │
│ p-primes-smart-500k     			 │      3125.96 		    │      325.40 		   │   10 	       │
└─────────────────────────┴──────────────┴─────────────┴──────┘
```

### Round 1 Winner: Claude (24.90 ms)

**LLM Rankings:**
1. Claude — 24.90 ms (fastest, lowest variance)
2. GPetey — 25.20 ms (+1.2%)
3. Grok — 25.57 ms (+2.7%)
4. Gemini — 29.66 ms (+19.1%)

---

## Round 2: n = 1,000,000,000 (50,847,534 primes)

### Non-Segmented Implementations

```
┌──────────────────────────┬──────────────┬─────────────┬─────────────┬─────────────┐
│ Implementation           				     │ Avg (ms)     		       │ Min (ms)    		      │ Max (ms)    		     │ StdDev (ms) 	    │
├──────────────────────────┼──────────────┼─────────────┼─────────────┼─────────────┤
│ r-primes-fast-1e9        			    │     2,265.82 		       │    2,156.45 		      │    2,579.73 		     │      115.03 		    │
│ c-primes-fast-claude-1e9		    │     3,170.10 		       │    3,085.33 		      │    3,300.26 		     │       52.59 		    │
│ c-primes-fast-1e9        			    │     3,719.54 		       │    3,612.04 		      │    4,013.30 		     │      115.38 		    │
│ c-primes-fast-gpetey-1e9 		    │     3,757.17 		       │    3,633.55 		      │    3,932.95 		     │       90.78 		    │
├──────────────────────────┼──────────────┼─────────────┼─────────────┼─────────────┤
│ c-primes-1e9 (baseline)  		    │    16,737.74 		       │   15,433.22 		      │   17,458.00 		     │      525.27 		    │
│ r-primes-1e9 (baseline)  		    │    19,458.01 		       │   14,370.34 		      │   20,654.37 		     │    1,716.09 		    │
└──────────────────────────┴──────────────┴─────────────┴─────────────┴─────────────┘
```

### Segmented Implementations

```
┌─────────────────────────────┬──────────────┬─────────────┬─────────────┬─────────────┐
│ Implementation              				     │ Avg (ms)     		        │ Min (ms)    		       │ Max (ms)    		      │ StdDev (ms) 	    │
├─────────────────────────────┼──────────────┼─────────────┼─────────────┼─────────────┤
│ c-primes-fast-gpetey-seg-1e9		     │       928.52 		        │      897.32 		       │      983.53 		      │       22.15 		    │	
│ c-primes-fast-claude-seg-1e9		     │     1,143.79 		        │    1,129.32 		       │    1,167.19 		      │       11.76 		    │
└─────────────────────────────┴──────────────┴─────────────┴─────────────┴─────────────┘
```

### Round 2 Winner: GPetey Segmented (928.52 ms)

**LLM Rankings (Segmented):**
1. GPetey — 928.52 ms (fastest)
2. Claude — 1,143.79 ms (+23.2%, but lowest variance)

---

## Round Aux: Specialised Implementations (n = 1,000,000,000)

```
┌──────────────────────────┬──────────────┬─────────────┬──────┐
│ Implementation           				    │ Mean Avg (ms)	       │ Std Dev (ms)	      │ Runs 	 │
├──────────────────────────┼──────────────┼─────────────┼──────┤
│ c-primes-the-beast-1e9   		     │        12.39 		       │        0.30 		      │   10 		 │
│ c-primes-bitpacked-1e9   		     │        12.60 		       │        0.72 		      │   10 		 │
│ c-primes-parallel-1e9    		     │        12.65 		       │        0.87 		      │   10		 │
│ r-primes-wheel-1e9       			     │        12.99 		       │        1.01 		      │   10 		 │
└──────────────────────────┴──────────────┴─────────────┴──────┘
```

*Note: These implementations output prime count only, not full prime list.*

---

## Round SIMD: Vectorised Implementations (n = 1,000,000,000)

```
┌──────────────────────────┬──────────────┬─────────────┬──────┐
│ Implementation           				    │ Mean Avg (ms)	       │ Std Dev (ms)	      │ Runs 	 │
├──────────────────────────┼──────────────┼─────────────┼──────┤
│ c-primes-parallel-1e9    		     │       719.43 		       │       16.27 		      │   10 		 │
│ c-primes-simd-1e9        			     │     1,448.90 		       │       17.12 		      │   10 		 │
└──────────────────────────┴──────────────┴─────────────┴──────┘
```

**Key Insight:** Multi-threading (719 ms) beats SIMD-only (1,449 ms) by 2x.
Combined SIMD + Parallel achieves best results.

---

## Summary: Final Standings

### By Implementation Type

```
┌────────────────────────────────┬──────────────┬───────────────────┐
│ Category                       						      │ Best Time    		        │ Winner            			        │
├────────────────────────────────┼──────────────┼───────────────────┤
│ Round 1 (n=500K)               					      │     24.90 ms 		        │ Claude            			        │
│ Round 2 Non-Seg (n=1B)         				      │  2,265.82 ms 	        │ Rust Fast         			        │
│ Round 2 Segmented (n=1B)       				      │    928.52 ms 		        │ GPetey           			        │
│ Parallel Only (n=1B)           				      │    719.43 ms 		        │ Multi-threaded    		        │
│ SIMD Only (n=1B)               					      │  1,448.90 ms 	        │ AVX2              				        │
│ Specialised Count-Only (n=1B)  			      │     12.39 ms 		        │ The Beast         			        │
└────────────────────────────────┴──────────────┴───────────────────┘
```

### LLM Head-to-Head

```
┌─────────┬────────────┬────────────┬─────────────┬────────────────────┐
│ LLM     		     │ Round 1    		│ Round 2    		      │ First-Try   	     │ Code Quality       		        │
│         		     │ (500K)     		│ (1B Seg)   		      │ Success     		     │                    					        │
├─────────┼────────────┼────────────┼─────────────┼────────────────────┤
│ Claude  	     │ 24.90 ms   		│ 1,143.79ms 	      │ Yes         			     │ Compact, efficient 	        │
│ GPetey  	     │ 25.20 ms   		│   928.52ms 	      │ Nearly      		     │ Elegant, readable  		        │
│ Grok    	     │ 25.57 ms   		│     —      			      │ No (4 iter) 	     │ Functional         			        │
│ Gemini  	     │ 29.66 ms   		│     —      			      │ No (3 iter) 	     │ Verbose            			        │
└─────────┴────────────┴────────────┴─────────────┴────────────────────┘
```

### Final Scores (30 points possible)

```
┌─────────┬───────┬──────────┬─────────┬───────┐
│ LLM     		     │ Speed 	   │ Compact  	  │ Elegant 	       │ Total        │
├─────────┼───────┼──────────┼─────────┼───────┤
│ Claude  	     │  10   		   │    10    		 │    0    		       │  20   	     │
│ GPetey  	     │   5   		   │     0    			 │   10    		       │  15   	     │
│ Grok    	     │   3   		   │     0    			 │    0    		       │   3   		     │
│ Gemini  	     │   0   		   │     0    			 │    0    		       │   0   		     │
└─────────┴───────┴──────────┴─────────┴───────┘
```

---

## Key Findings

1. **Claude** produced the most compact code (18 lines) with zero compilation errors
2. **GPetey** produced the most elegant/readable code with fastest segmented performance
3. **Grok** and **Gemini** required multiple iterations to achieve working code
4. **Parallelism** provides 2x speedup over SIMD alone at this scale
5. **Segmentation** is essential for billion-scale — 4x faster than non-segmented
6. All LLMs used the **Sieve of Eratosthenes** — no novel algorithms emerged

---

*Benchmarks conducted December 2025*
*Full source code: github.com/whisprer/prime-shootout*
