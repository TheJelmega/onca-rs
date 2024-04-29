use crate::*;

/// Pass that can either optimize or process a regex
pub trait OptProcessPass {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError>;

    fn process_multi(&self, processor: &RegexProcessor, nodes: &mut Vec<RegexNode>) -> Result<(), RegexError> {
        for node in nodes {
            self.process_single(processor, node)?;
        }
        Ok(())
    }
}

pub(crate) struct RegexProcessor {
    
}

impl RegexProcessor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn process_and_optimize(&self, node: &mut RegexNode) -> Result<(), RegexError> {
        // OPT PASS 1: Literal combine
		// (?:a)(?:\d)(?:a) -> (?:a\da)
		//
		// We do this as early as possible to decrease number of nodes that need to be visited later on
        self.process_nested(node, &LiteralCombinePass)?;

        // OPT PASS 2: Eliminate None elements
		// These are nodes emitted during parsing if the node has a meaning for that process, but not for matching
        self.process_nested(node, &NoneElimination)?;

        // PROCESS PASS 1: Split group with options in separate internal option node inside of group
		// (?im-sx:...) -> (?:(?im-sx)...)
        self.process_nested(node, &SplitGroupAndOptions)?;

        // PROCESS PASS 2: Process lookbehind and set nodes per alteration
        self.process_nested(node, &ResolveLookbehind)?;

        // PROCESS PASS 3: Processs `\K` by changing the capture
        // (foo\Kbar) -> (?:foo(bar))
        self.process_nested(node, &MatchRestartSplitter)?;
        
        // PROCESS PASS 3: go through nodes in reverse order to set repetition sub-groups
        self.process_nested(node, &ResolveRepetitionTail)?;

        Ok(())
    }

    fn process_nested<T: OptProcessPass>(&self, node: &mut RegexNode, pass: &T) -> Result<(), RegexError> {
        match node {
			RegexNode::Unit(nodes) => pass.process_multi(self, nodes),
			RegexNode::Alternation(alterations) => {
				for nodes in alterations {
					pass.process_multi(self, nodes)?;
				}
				Ok(())
			},
			RegexNode::Repetition(inner, _, _, _) => pass.process_single(self, inner),
			RegexNode::Lookahead(inner, _) => pass.process_single(self, inner),
			RegexNode::Lookbehind(nodes, _, _) => pass.process_multi(self, nodes),
			RegexNode::AbsConditional(_, yes, no) => {
				pass.process_single(self, &mut *yes)?;
				if let Some(no) = no {
					pass.process_single(self, &mut *no)?;
				}
				Ok(())
			},
			RegexNode::NamedConditional(_, yes, no) => {
				pass.process_single(self, &mut *yes)?;
				if let Some(no) = no {
					pass.process_single(self, &mut *no)?;
				}
				Ok(())
			},
			RegexNode::RecursiveConditional(_, yes, no) => {
				pass.process_single(self, &mut *yes)?;
				if let Some(no) = no {
					pass.process_single(self, &mut *no)?;
				}
				Ok(())
			},
			RegexNode::NamedRecursiveConditional(_, yes, no) => {
				pass.process_single(self, &mut *yes)?;
				if let Some(no) = no {
					pass.process_single(self, &mut *no)?;
				}
				Ok(())
			},
			RegexNode::DefineConditional(yes) => pass.process_single(self, &mut *yes),
			RegexNode::AssertConditional(assert, yes, no) => {
				pass.process_single(self, &mut *assert)?;
				pass.process_single(self, &mut *yes)?;
				if let Some(no) = no {
					pass.process_single(self, &mut *no)?;
				}
				Ok(())
			},
			RegexNode::ParsedGroup(_, _, inner, _) => pass.process_single(self, inner),
			RegexNode::Group { sub_node, .. } => pass.process_single(self, sub_node),
			_ => Ok(()),
		}
    }
}

struct LiteralCombinePass;

impl OptProcessPass for LiteralCombinePass {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError> {
        processor.process_nested(node, self)
    }
    
    fn process_multi(&self, processor: &RegexProcessor, nodes: &mut Vec<RegexNode>) -> Result<(), RegexError> {
        let mut idx = 0;
		let mut i = 0;

		// Reserve the minimum amount of memory possible
		let mut buffer = String::with_capacity(1);

		let process_lit_node = |nodes: &mut Vec<RegexNode>, buffer: &mut String, idx: usize, i: usize| if !buffer.is_empty() && idx + 1 < i {
			if buffer.chars().nth(2).is_some() {
				nodes[idx] = RegexNode::Literal(buffer.clone());
			} else {
				nodes[idx] = RegexNode::LiteralChar(buffer.chars().next().unwrap());
			}
			buffer.clear();
			if i > idx {
				nodes.drain(idx+1..i);
			}
		};

		for _ in 0..nodes.len() {
			// We modify the actual nodes, so make sure not to go out of bounds after removing nodes
			if i >= nodes.len() {
				break;
			}

			match &mut nodes[i] {
				RegexNode::Literal(lit) => {
					buffer.push_str(&lit);
				},
				RegexNode::LiteralChar(ch) => {
					buffer.push(*ch);
				},
				node => {
					// Handle inner nodes
					processor.process_nested(node, self)?;

					process_lit_node(nodes, &mut buffer, idx, i);
					i = idx;
					idx += 1;
				},
			}
			i += 1;
		}
		process_lit_node(nodes, &mut buffer, idx, i);
        Ok(())
    }
}

struct NoneElimination;

impl OptProcessPass for NoneElimination {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError> {
        processor.process_nested(node, self)
    }

    fn process_multi(&self, processor: &RegexProcessor, nodes: &mut Vec<RegexNode>) -> Result<(), RegexError> {
        nodes.retain(|node| !matches!(node, RegexNode::None));
        for node in nodes {
            self.process_single(processor, node)?;
        }
        Ok(())
    }
}

struct SplitGroupAndOptions;

impl OptProcessPass for SplitGroupAndOptions {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError> {
        processor.process_nested(node, self)?;

        if let RegexNode::ParsedGroup(options, capture_idx, inner, atomic) = node {
			let inner = core::mem::replace(inner, Box::new(RegexNode::None));

			let mut sub_nodes = Vec::with_capacity(2);
            if options.is_any() {
                sub_nodes.push(RegexNode::InternalOptionSetting(*options));
            }
			sub_nodes.push(*inner);

			*node = RegexNode::Group { capture_idx: *capture_idx, sub_node: Box::new(RegexNode::Unit(sub_nodes)), atomic: *atomic };
		}
		Ok(())
    }
}

struct ResolveLookbehind;

impl OptProcessPass for ResolveLookbehind {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError> {
        processor.process_nested(node, self)?;

		if let RegexNode::Lookbehind(lookbehind_nodes, node_lens, _) = node {
			// Step 1: expand top-level alteration if needed
			if lookbehind_nodes.len() == 1 {
				if let RegexNode::Alternation(_) = &lookbehind_nodes[0] {
					let RegexNode::Alternation(alteration_nodes) = core::mem::replace(&mut lookbehind_nodes[0], RegexNode::None) else { unreachable!() };
					let mut inner = Vec::with_capacity(alteration_nodes.len());
					for alteration in alteration_nodes {
						inner.push(RegexNode::Unit(alteration));
					}
					*lookbehind_nodes = inner;
				}
			}
			// Step 2: Calculate fixed lenghts
			for node in lookbehind_nodes {
				let Some(len) = node.get_fixed_length() else { return Err(RegexError::new_str("Lookbehind contains an element with an invalid length", 0, 0)) };
				node_lens.push(len);
			}
		}
		Ok(())
    }
}

struct ResolveRepetitionTail;

impl OptProcessPass for ResolveRepetitionTail {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError> {
        processor.process_nested(node, self)
    }

    fn process_multi(&self, processor: &RegexProcessor, nodes: &mut Vec<RegexNode>) -> Result<(), RegexError> {
        for node in nodes.iter_mut() {
            self.process_single(processor, node)?;
        }

        for i in (0..nodes.len() - 1).rev() {
            if matches!(nodes[i], RegexNode::Repetition(..)) {
                let tail = nodes.split_off(i + 1);
                let RegexNode::Repetition(_, tail_ref, _, _) = &mut nodes[i] else { unreachable!() };
                *tail_ref = tail;
            }
        }

        Ok(())
    }
}

struct MatchRestartSplitter;

impl OptProcessPass for  MatchRestartSplitter {
    fn process_single(&self, processor: &RegexProcessor, node: &mut RegexNode) -> Result<(), RegexError> {
        processor.process_nested(node, self)?;

        if let RegexNode::Group { capture_idx, sub_node, atomic } = node {
            // A group either has a unit or an alteration nested in it
            if let RegexNode::Unit(inner) = &mut **sub_node {
                // Find last occurance of `\K` and split there
                if let Some((idx, _)) = inner.iter().enumerate().rev().find(|(_, node)| matches!(node, RegexNode::MatchStartReset)) {
                    let capture_nodes = inner.split_off(idx);
                    inner.push(RegexNode::Group { capture_idx: *capture_idx, sub_node: Box::new(RegexNode::Unit(capture_nodes)), atomic: *atomic });
                    *capture_idx = None;
                }

            } else {
                return Err(RegexError::new_str("A match restart is not supported when nested in an alteration", 0, 0))
            }
        }
        Ok(())
    }
}