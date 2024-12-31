import shucker

def test_shucker():
    assert shucker.shuck("http://foo/bar") == "http://foo/bar"