import nltk
from nltk.tokenize import word_tokenize
from nltk.corpus import stopwords

class LanguageEnhancer:
    def __init__(self):
        self.stop_words = set(stopwords.words('english'))

    def enhance_text(self, text):
        tokens = word_tokenize(text)
        tokens = [t for t in tokens if t.lower() not in self.stop_words]
        return ' '.join(tokens)

# Usage example
enhancer = LanguageEnhancer()
text = "This is an example sentence with some stop words."
enhanced_text = enhancer.enhance_text(text)
print(enhanced_text)