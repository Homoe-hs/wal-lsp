package tree_sitter_wal_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-wal"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_wal.Language())
	if language == nil {
		t.Errorf("Error loading Wal grammar")
	}
}
