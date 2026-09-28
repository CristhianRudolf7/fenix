import unittest
import threading
from openai_decision_proxy import Quotas, seconds


class QuotaTests(unittest.TestCase):
    def test_waiting_large_request_is_not_starved_by_smaller_requests(self):
        now = [0]
        q = Quotas(['shared'], clock=lambda: now[0])
        q.reserve(150000)
        waiting, refill, small_started = threading.Event(), threading.Event(), threading.Event()
        order = []

        def sleep(delay):
            waiting.set()
            if not refill.wait(2):
                raise TimeoutError('Test did not release the quota refill')
            now[0] += delay

        def large():
            order.append(('large', q.acquire_estimate(150000, sleep)))

        def small():
            small_started.set()
            order.append(('small', q.acquire_estimate(1000, sleep)))

        first = threading.Thread(target=large)
        second = threading.Thread(target=small)
        first.start()
        self.assertTrue(waiting.wait(2))
        second.start()
        self.assertTrue(small_started.wait(2))
        self.assertEqual(order, [])
        refill.set()
        first.join(2)
        second.join(2)
        self.assertFalse(first.is_alive() or second.is_alive())
        self.assertEqual([name for name, _ in order], ['large', 'small'])
        self.assertEqual(order[0][1], (0, 150000))

    def test_independent_accounts_can_reserve_in_parallel(self):
        q=Quotas(['a','b'],clock=lambda:0)
        self.assertEqual(q.reserve(150000),(0,0))
        self.assertEqual(q.reserve(150000),(1,0))
        self.assertEqual(q.reserve(150000),(None,30))

    def test_oversized_wire_estimate_waits_for_full_bucket_without_local_rejection(self):
        q = Quotas(['shared', 'shared'], clock=lambda: 0)
        self.assertEqual(q.reserve_estimate(300000), (0, 0, 200000))
        self.assertEqual(q.reserve_estimate(300000), (None, 60, 200000))
        q.observe(0, {'retry-after': '90'}, 429)
        self.assertEqual(q.reserve_estimate(300000), (None, 90, 200000))

    def test_estimate_respects_reduced_provider_limit(self):
        q = Quotas(['one'], clock=lambda: 0)
        q.observe(0, {'x-ratelimit-limit-tokens': '100000',
                      'x-ratelimit-remaining-tokens': '100000'}, 200)
        self.assertEqual(q.reserve_estimate(200000), (0, 0, 100000))
        self.assertRaises(ValueError, q.reserve, 100001)

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
