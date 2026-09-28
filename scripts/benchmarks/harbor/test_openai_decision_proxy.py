import unittest
from openai_decision_proxy import Quotas, seconds


class QuotaTests(unittest.TestCase):
    def test_independent_accounts_can_reserve_in_parallel(self):
        q=Quotas(['a','b'],clock=lambda:0)
        self.assertEqual(q.reserve(150000),(0,0))
        self.assertEqual(q.reserve(150000),(1,0))
        self.assertEqual(q.reserve(150000),(None,30))

    def test_same_organization_does_not_double_quota(self):
        q=Quotas(['shared','shared'],clock=lambda:0)
        self.assertEqual(q.reserve(150000),(0,0))
        self.assertEqual(q.reserve(150000),(None,30))

    def test_429_cools_down_the_entire_group(self):
        q=Quotas(['a','a'],clock=lambda:0)
        q.observe(0,{'retry-after':'12','x-ratelimit-reset-tokens':'1m2s'},429)
        self.assertEqual(q.reserve(1),(None,62))

    def test_refills_with_time_and_respects_observed_remaining(self):
        now=[0];q=Quotas(['a'],clock=lambda:now[0])
        q.observe(0,{'x-ratelimit-limit-tokens':'100000','x-ratelimit-remaining-tokens':'0'},200)
        self.assertEqual(q.reserve(50000),(None,30))
        now[0]=30
        self.assertEqual(q.reserve(50000),(0,0))
        self.assertRaises(ValueError,q.reserve,100001)
        self.assertAlmostEqual(seconds('1m2s500ms'),62.5)

if __name__=='__main__':unittest.main()
