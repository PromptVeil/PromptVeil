import unittest

class TestImport(unittest.TestCase):
    def test_import(self):
        try:
            import promptveil
            import promptveil_core
            self.assertTrue(True)
        except ImportError as e:
            self.fail(f"Failed to import modules: {str(e)}")

if __name__ == '__main__':
    unittest.main() 