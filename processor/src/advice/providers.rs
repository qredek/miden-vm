use super::{
    AdviceInputs, AdviceProvider, AdviceSource, BTreeMap, ExecutionError, Felt, IntoBytes, KvMap,
    MerklePath, MerkleStore, NodeIndex, RecordingMap, RpoDigest, StarkField, StoreNode, Vec, Word,
};

// TYPE ALIASES
// ================================================================================================

type SimpleMerkleMap = BTreeMap<RpoDigest, StoreNode>;
type RecordingMerkleMap = RecordingMap<RpoDigest, StoreNode>;

type SimpleAdviceMap = BTreeMap<[u8; 32], Vec<Felt>>;
type RecordingAdviceMap = RecordingMap<[u8; 32], Vec<Felt>>;

// BASE ADVICE PROVIDER
// ================================================================================================

/// An in-memory [AdviceProvider] implementation which serves as the base for advice providers
/// bundles with Miden VM.
#[derive(Debug, Clone, Default)]
pub struct BaseAdviceProvider<M, S>
where
    M: KvMap<[u8; 32], Vec<Felt>>,
    S: KvMap<RpoDigest, StoreNode>,
{
    step: u32,
    stack: Vec<Felt>,
    map: M,
    store: MerkleStore<S>,
}

impl<M, S> From<AdviceInputs> for BaseAdviceProvider<M, S>
where
    M: KvMap<[u8; 32], Vec<Felt>>,
    S: KvMap<RpoDigest, StoreNode>,
{
    fn from(inputs: AdviceInputs) -> Self {
        let (mut stack, map, store) = inputs.into_parts();
        stack.reverse();
        Self {
            step: 0,
            stack,
            map: map.into_iter().collect(),
            store: store.inner_nodes().collect(),
        }
    }
}

impl<M, S> AdviceProvider for BaseAdviceProvider<M, S>
where
    M: KvMap<[u8; 32], Vec<Felt>>,
    S: KvMap<RpoDigest, StoreNode>,
{
    // ADVICE STACK
    // --------------------------------------------------------------------------------------------

    fn pop_stack(&mut self) -> Result<Felt, ExecutionError> {
        self.stack.pop().ok_or(ExecutionError::AdviceStackReadFailed(self.step))
    }

    fn pop_stack_word(&mut self) -> Result<Word, ExecutionError> {
        if self.stack.len() < 4 {
            return Err(ExecutionError::AdviceStackReadFailed(self.step));
        }

        let idx = self.stack.len() - 4;
        let result =
            [self.stack[idx + 3], self.stack[idx + 2], self.stack[idx + 1], self.stack[idx]];

        self.stack.truncate(idx);

        Ok(result)
    }

    fn pop_stack_dword(&mut self) -> Result<[Word; 2], ExecutionError> {
        let word0 = self.pop_stack_word()?;
        let word1 = self.pop_stack_word()?;

        Ok([word0, word1])
    }

    fn push_stack(&mut self, source: AdviceSource) -> Result<(), ExecutionError> {
        match source {
            AdviceSource::Value(value) => {
                self.stack.push(value);
                Ok(())
            }

            AdviceSource::Map { key, include_len } => {
                let values = self
                    .map
                    .get(&key.into_bytes())
                    .ok_or(ExecutionError::AdviceKeyNotFound(key))?;

                self.stack.extend(values.iter().rev());
                if include_len {
                    self.stack.push(Felt::from(values.len() as u64));
                }
                Ok(())
            }
        }
    }

    fn insert_into_map(&mut self, key: Word, values: Vec<Felt>) -> Result<(), ExecutionError> {
        self.map.insert(key.into_bytes(), values);
        Ok(())
    }

    // ADVISE SETS
    // --------------------------------------------------------------------------------------------

    fn get_tree_node(
        &self,
        root: Word,
        depth: &Felt,
        index: &Felt,
    ) -> Result<Word, ExecutionError> {
        let index = NodeIndex::from_elements(depth, index).map_err(|_| {
            ExecutionError::InvalidTreeNodeIndex {
                depth: *depth,
                value: *index,
            }
        })?;
        self.store
            .get_node(root.into(), index)
            .map(|v| v.into())
            .map_err(ExecutionError::MerkleStoreLookupFailed)
    }

    fn get_merkle_path(
        &self,
        root: Word,
        depth: &Felt,
        index: &Felt,
    ) -> Result<MerklePath, ExecutionError> {
        let index = NodeIndex::from_elements(depth, index).map_err(|_| {
            ExecutionError::InvalidTreeNodeIndex {
                depth: *depth,
                value: *index,
            }
        })?;
        self.store
            .get_path(root.into(), index)
            .map(|value| value.path)
            .map_err(ExecutionError::MerkleStoreLookupFailed)
    }

    fn get_leaf_depth(
        &self,
        root: Word,
        tree_depth: &Felt,
        index: &Felt,
    ) -> Result<u8, ExecutionError> {
        let tree_depth = u8::try_from(tree_depth.as_int())
            .map_err(|_| ExecutionError::InvalidTreeDepth { depth: *tree_depth })?;
        self.store
            .get_leaf_depth(root.into(), tree_depth, index.as_int())
            .map_err(ExecutionError::MerkleStoreLookupFailed)
    }

    fn update_merkle_node(
        &mut self,
        root: Word,
        depth: &Felt,
        index: &Felt,
        value: Word,
    ) -> Result<MerklePath, ExecutionError> {
        let node_index = NodeIndex::from_elements(depth, index).map_err(|_| {
            ExecutionError::InvalidTreeNodeIndex {
                depth: *depth,
                value: *index,
            }
        })?;
        self.store
            .set_node(root.into(), node_index, value.into())
            .map(|root| root.path)
            .map_err(ExecutionError::MerkleStoreUpdateFailed)
    }

    fn merge_roots(&mut self, lhs: Word, rhs: Word) -> Result<Word, ExecutionError> {
        self.store
            .merge_roots(lhs.into(), rhs.into())
            .map(|v| v.into())
            .map_err(ExecutionError::MerkleStoreMergeFailed)
    }

    // CONTEXT MANAGEMENT
    // --------------------------------------------------------------------------------------------

    fn advance_clock(&mut self) {
        self.step += 1;
    }
}

// MEMORY ADVICE PROVIDER
// ================================================================================================

/// An in-memory `[AdviceProvider]` implementation which uses [BTreeMap]s as its backing storage.
#[derive(Debug, Clone, Default)]
pub struct MemAdviceProvider {
    provider: BaseAdviceProvider<SimpleAdviceMap, SimpleMerkleMap>,
}

impl From<AdviceInputs> for MemAdviceProvider {
    fn from(inputs: AdviceInputs) -> Self {
        let provider = inputs.into();
        Self { provider }
    }
}

/// Accessors to internal data structures of the provider used for testing purposes.
#[cfg(any(test, feature = "internals"))]
impl MemAdviceProvider {
    /// Returns the current state of the advice stack.
    pub fn stack(&self) -> &[Felt] {
        &self.provider.stack
    }

    /// Returns the current state of the advice map.
    pub fn map(&self) -> &SimpleAdviceMap {
        &self.provider.map
    }

    // Returns the current state of the Merkle store.
    pub fn store(&self) -> &MerkleStore<SimpleMerkleMap> {
        &self.provider.store
    }

    /// Returns true if the Merkle root exists for the advice provider Merkle store.
    pub fn has_merkle_root(&self, root: crate::crypto::RpoDigest) -> bool {
        self.provider.store.get_node(root, NodeIndex::root()).is_ok()
    }
}

/// Pass-through implementations of [AdviceProvider] methods.
/// 
/// TODO: potentially do this via a macro.
#[rustfmt::skip]
impl AdviceProvider for MemAdviceProvider {
    fn pop_stack(&mut self) -> Result<Felt, ExecutionError> {
        self.provider.pop_stack()
    }

    fn pop_stack_word(&mut self) -> Result<Word, ExecutionError> {
        self.provider.pop_stack_word()
    }

    fn pop_stack_dword(&mut self) -> Result<[Word; 2], ExecutionError> {
        self.provider.pop_stack_dword()
    }

    fn push_stack(&mut self, source: AdviceSource) -> Result<(), ExecutionError> {
        self.provider.push_stack(source)
    }

    fn insert_into_map(&mut self, key: Word, values: Vec<Felt>) -> Result<(), ExecutionError> {
        self.provider.insert_into_map(key, values)
    }

    fn get_tree_node(&self, root: Word, depth: &Felt, index: &Felt) -> Result<Word, ExecutionError> {
        self.provider.get_tree_node(root, depth, index)
    }

    fn get_merkle_path(&self, root: Word, depth: &Felt, index: &Felt) -> Result<MerklePath, ExecutionError> {
        self.provider.get_merkle_path(root, depth, index)
    }

    fn get_leaf_depth(&self, root: Word, tree_depth: &Felt, index: &Felt) -> Result<u8, ExecutionError> {
        self.provider.get_leaf_depth(root, tree_depth, index)
    }

    fn update_merkle_node(&mut self, root: Word, depth: &Felt, index: &Felt, value: Word) -> Result<MerklePath, ExecutionError> {
        self.provider.update_merkle_node(root, depth, index, value)
    }

    fn merge_roots(&mut self, lhs: Word, rhs: Word) -> Result<Word, ExecutionError> {
        self.provider.merge_roots(lhs, rhs)
    }

    fn advance_clock(&mut self) {
        self.provider.advance_clock()
    }
}

// RECORDING ADVICE PROVIDER
// ================================================================================================

/// An in-memory `[AdviceProvider]` implementation with support for data access recording.
///
/// The recorder can be converted into a proof which can be used to provide the non-deterministic
/// inputs for program execution.
#[derive(Debug, Clone, Default)]
pub struct RecAdviceProvider {
    provider: BaseAdviceProvider<RecordingAdviceMap, RecordingMerkleMap>,
    init_stack: Vec<Felt>,
}

impl RecAdviceProvider {
    /// Consumes the advice provider and returns a [AdviceInputs] instance which can be used to
    /// re-execute the program.
    ///
    /// The returned [AdviceInputs] instance will contain only the non-deterministic inputs which
    /// were requested during program execution.
    pub fn into_proof(self) -> AdviceInputs {
        let Self {
            provider,
            init_stack,
        } = self;
        let BaseAdviceProvider {
            step: _,
            stack: _,
            map,
            store,
        } = provider;

        let map = map.into_proof();
        let store = store.into_inner().into_proof();

        AdviceInputs::default()
            .with_stack(init_stack)
            .with_map(map)
            .with_merkle_store(store.into())
    }
}

impl From<AdviceInputs> for RecAdviceProvider {
    fn from(inputs: AdviceInputs) -> Self {
        let init_stack = inputs.stack().to_vec();
        let provider = inputs.into();
        Self {
            provider,
            init_stack,
        }
    }
}

/// Accessors to internal data structures of the provider used for testing purposes.
#[cfg(any(test, feature = "internals"))]
impl RecAdviceProvider {
    /// Returns the current state of the advice stack.
    pub fn stack(&self) -> &[Felt] {
        &self.provider.stack
    }

    /// Returns the current state of the advice map.
    pub fn map(&self) -> &RecordingAdviceMap {
        &self.provider.map
    }

    // Returns the current state of the Merkle store.
    pub fn store(&self) -> &MerkleStore<RecordingMerkleMap> {
        &self.provider.store
    }

    /// Returns true if the Merkle root exists for the advice provider Merkle store.
    pub fn has_merkle_root(&self, root: crate::crypto::RpoDigest) -> bool {
        self.provider.store.get_node(root, NodeIndex::root()).is_ok()
    }
}

/// Pass-through implementations of [AdviceProvider] methods.
/// 
/// TODO: potentially do this via a macro.
#[rustfmt::skip]
impl AdviceProvider for RecAdviceProvider {
    fn pop_stack(&mut self) -> Result<Felt, ExecutionError> {
        self.provider.pop_stack()
    }

    fn pop_stack_word(&mut self) -> Result<Word, ExecutionError> {
        self.provider.pop_stack_word()
    }

    fn pop_stack_dword(&mut self) -> Result<[Word; 2], ExecutionError> {
        self.provider.pop_stack_dword()
    }

    fn push_stack(&mut self, source: AdviceSource) -> Result<(), ExecutionError> {
        self.provider.push_stack(source)
    }

    fn insert_into_map(&mut self, key: Word, values: Vec<Felt>) -> Result<(), ExecutionError> {
        self.provider.insert_into_map(key, values)
    }

    fn get_tree_node(&self, root: Word, depth: &Felt, index: &Felt) -> Result<Word, ExecutionError> {
        self.provider.get_tree_node(root, depth, index)
    }

    fn get_merkle_path(&self, root: Word, depth: &Felt, index: &Felt) -> Result<MerklePath, ExecutionError> {
        self.provider.get_merkle_path(root, depth, index)
    }

    fn get_leaf_depth(&self, root: Word, tree_depth: &Felt, index: &Felt) -> Result<u8, ExecutionError> {
        self.provider.get_leaf_depth(root, tree_depth, index)
    }

    fn update_merkle_node(&mut self, root: Word, depth: &Felt, index: &Felt, value: Word) -> Result<MerklePath, ExecutionError> {
        self.provider.update_merkle_node(root, depth, index, value)
    }

    fn merge_roots(&mut self, lhs: Word, rhs: Word) -> Result<Word, ExecutionError> {
        self.provider.merge_roots(lhs, rhs)
    }

    fn advance_clock(&mut self) {
        self.provider.advance_clock()
    }
}
