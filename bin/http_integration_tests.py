import json
import logging
import unittest

import requests


class WorkAPITests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        logging.basicConfig()
        logging.getLogger().setLevel(logging.DEBUG)
        reqs_log_cfg = logging.getLogger("requests.packages.urllib3")
        reqs_log_cfg.setLevel(logging.DEBUG)
        reqs_log_cfg.propagate = True

    def setUp(self):
        print("\n")

    # `curl -i -X POST localhost:3000/work`
    def test_create_work(self):
        # given
        url = "http://localhost:3000/work"
        # when
        res = requests.post(url)
        # then
        self.assertEqual(res.status_code, 200)
        WorkAPITests.new_work = json.loads(res.text)
        logging.info(f"We created this work: {WorkAPITests.new_work}")

    # `curl -i -X GET localhost:3000/work/1000`
    def test_retrieve_work(self):
        # given
        url = f"http://localhost:3000/work/{WorkAPITests.new_work['id']}"
        # when
        res = requests.get(url)
        # then
        self.assertEqual(res.status_code, 200)
        self.assertDictEqual(WorkAPITests.new_work, json.loads(res.text))
        logging.info(f"We retrieved this same work: {WorkAPITests.new_work}")

    # `curl -i -X GET localhost:3000/work/search?work_code=foo`
    def test_search_work(self):
        # given
        url = f"http://localhost:3000/work/search?work_code={WorkAPITests.new_work['work_code']}"
        # when
        res = requests.get(url)
        # then
        self.assertEqual(res.status_code, 200)
        work_list = json.loads(res.text)
        self.assertEqual(len(work_list), 1)
        self.assertDictEqual(WorkAPITests.new_work, work_list[0])
        logging.info(
            f"We searched for and retrieved this same work: {WorkAPITests.new_work}"
        )


if __name__ == "__main__":
    unittest.main()
