#include<bits/stdc++.h>
using u=uint64_t;int main(){u n=500000,h=n/2,i,j,w;std::vector<u>b((h>>6)+1,~0ULL),r{2};
for(b[0]^=1,i=1;i<=sqrt(n)/2;++i)if(b[i>>6]>>(i&63)&1)for(j=2*i*(i+1);j<=h;j+=2*i+1)b[j
>>6]&=~(1ULL<<(j&63));for(i=0;i<b.size();++i)for(w=b[i];w;w&=w-1)if(u p=((i<<6)+__builtin_ctzll
(w))*2+1;p<=n)r.push_back(p);std::cout<<r.size()<<" primes\n";}