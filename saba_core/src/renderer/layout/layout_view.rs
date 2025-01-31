use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};

use crate::{ constants::CONTENT_AREA_WIDTH, display_item::DisplayItem, renderer::{css::cssom::StyleSheet, dom::{api::get_target_element_node, node::{ElementKind, Node}}}};

use super::layout_object::{  create_layout_object, LayoutObject, LayoutObjectKind, LayoutPoint, LayoutSize};



#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn new(root: Rc<RefCell<Node>>, cssom: &StyleSheet) -> Self {
        let body_root = get_target_element_node(Some(root), ElementKind::Body);
        let mut tree = Self {
            root: build_layout_tree(&body_root, &None, cssom),
        };
        tree.update_layout();
        tree
    }

    pub fn root(&self) -> Option<Rc<RefCell<LayoutObject>>> {
        self.root.clone()
    }

    /** 構築し終えたレイアウトツリーに対して、各ノードのサイズと位置を計算する */
    fn update_layout(&mut self) {
        Self::calculate_node_size(&self.root, LayoutSize::new(CONTENT_AREA_WIDTH, 0));
        Self::calculate_node_position(&self.root, LayoutPoint::new(0, 0), LayoutObjectKind::Block, None, None);
        
    }

    /** レイアウトツリーの各ノードのサイズを再帰的に計算する */
    fn calculate_node_size(node: &Option<Rc<RefCell<LayoutObject>>>, parent_size: LayoutSize) {
       if let Some(n) = node {
           if n.borrow().kind() == LayoutObjectKind::Block {
            n.borrow_mut().compute_size(parent_size);
           }

           let first_child = n.borrow().first_child();
           Self::calculate_node_size(&first_child, n.borrow().size());

           let next_sibling = n.borrow().next_sibling();
           Self::calculate_node_size(&next_sibling, parent_size);
           
           n.borrow_mut().compute_size(parent_size);
       }
    }

    /** レイアウトツリーのノードの位置を再帰的に計算する */
    fn calculate_node_position(
        node: &Option<Rc<RefCell<LayoutObject>>>, 
        parent_point: LayoutPoint, 
        previous_sibling_kind: LayoutObjectKind, 
        previous_sibling_point: Option<LayoutPoint>, 
        previous_sibling_size: Option<LayoutSize>
    ) {
        if let Some(n) = node {
            n.borrow_mut().compute_position(parent_point, previous_sibling_kind, previous_sibling_point, previous_sibling_size);

            let first_child = n.borrow().first_child();
            Self::calculate_node_position(&first_child, n.borrow().point(), LayoutObjectKind::Block, None, None);

            let next_sibling = n.borrow().next_sibling();
            Self::calculate_node_position(&next_sibling, parent_point, n.borrow().kind(), Some(n.borrow().point()),Some( n.borrow().size()));
        }
    }

    fn paint_node(node: &Option<Rc<RefCell<LayoutObject>>>, display_items: &mut Vec<DisplayItem>) {
        match node {
            Some(n) => {
                // ノードの描画情報を取得し、描画情報リストに追加していく
                display_items.extend(n.borrow_mut().paint());

                // 子要素があれば再帰的に描画情報を取得する
                let first_child = n.borrow().first_child();
                Self::paint_node(&first_child, display_items);

                // 次の兄弟要素があれば再帰的に描画情報を取得する
                let next_sibling = n.borrow().next_sibling();
                Self::paint_node(&next_sibling, display_items);
            }
            None => ()
        }
    }

    pub fn paint(&self) -> Vec<DisplayItem> {
        let mut display_items = Vec::new();
        Self::paint_node(&self.root, &mut display_items);
        display_items
    }

    // 指定された位置がレイアウトツリーのどのノードを指しているかを見つける関数
    pub fn find_node_by_position(&self, position: (i64, i64)) -> Option<Rc<RefCell<LayoutObject>>> {
        Self::find_node_by_position_internal(&self.root(), position)
    }

    // 再帰的に実行される。クリックした位置にある葉ノード、つまり、木構造の下位の末端にあるノードを返す。
    // 大抵の場合テキストノードが返る
    fn find_node_by_position_internal(
        node: &Option<Rc<RefCell<LayoutObject>>>,
        position: (i64, i64)
    ) -> Option<Rc<RefCell<LayoutObject>>> {
        match node {
            Some(n) => {
                let first_child = n.borrow().first_child();
                let result1 = Self::find_node_by_position_internal(&first_child, position);
                if result1.is_some() {
                    return result1;
                }

                let next_sibling = n.borrow().next_sibling();
                let result2 = Self::find_node_by_position_internal(&next_sibling, position);
                
                if result2.is_some() {
                    return result2;
                }

                if n.borrow().point().x() <= position.0 
                && position.0 <= (n.borrow().point().x() + n.borrow().size().width()) 
                && n.borrow().point().y() <= position.1 
                && position.1 <= (n.borrow().point().y() + n.borrow().size().height()) 
                {
                    return Some(n.clone());
                }
                None
            }
            None => None,
        }
    }

}

fn build_layout_tree(node: &Option<Rc<RefCell<Node>>>, parent_obj: &Option<Rc<RefCell<LayoutObject>>>, cssom: &StyleSheet) -> Option<Rc<RefCell<LayoutObject>>>
{
   let mut target_node = node.clone();
   let mut layout_object = create_layout_object(node, parent_obj, cssom);

   while layout_object.is_none() {
        if let Some(n) = target_node {
            target_node = n.borrow().next_sibling().clone();
            layout_object = create_layout_object(&target_node, parent_obj, cssom);
        }else {
            return layout_object;
        }
    }

    if let Some(n) = target_node {
        let original_first_child = n.borrow().first_child();
        let original_next_sibling = n.borrow().next_sibling();
        let mut first_child = build_layout_tree(&original_first_child, &layout_object, cssom);
        let mut next_sibling = build_layout_tree(&original_next_sibling, &None, cssom);

        if first_child.is_none() && original_first_child.is_some() {
            let mut original_dom_node = original_first_child.expect("first child shud exist").borrow().next_sibling();

            loop {
                first_child = build_layout_tree(&original_dom_node, &layout_object, cssom);

                if first_child.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node.expect("next sibling should exist").borrow().next_sibling();
                    continue;
                }

                break;
                
            }
        }

        if next_sibling.is_none() && n.borrow().next_sibling().is_some() {
            let mut original_dom_node = original_next_sibling.expect("first child should exist").borrow().next_sibling();
            
            loop {
                next_sibling = build_layout_tree(&original_dom_node, &None, cssom);
            
                if next_sibling.is_none() && original_dom_node.is_some() {
                    original_dom_node = original_dom_node.expect("next sibling should exist").borrow().next_sibling();
                    continue;
                }
            
                break;
             }
        }

        let obj = match layout_object {
            Some(ref obj) => obj,
            None => panic!("render object should exist here"),
        };
        
        obj.borrow_mut().set_first_child(first_child);
        obj.borrow_mut().set_next_sibling(next_sibling);
            
    }

   layout_object
}



#[cfg(test)]
mod tests {
    use alloc::{string::{String, ToString}, vec::Vec};
    use crate::renderer::{css::{cssom::CssParser, token::CssTokenizer}, dom::{api::get_style_content, node::{Element, NodeKind}}, html::{parser::HtmlParser, token::HtmlTokenizer}, layout::layout_object::LayoutObjectKind};
    use super::LayoutView;

    fn create_layout_view(html: String) -> LayoutView {
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let dom = window.borrow().document();
        let style = get_style_content(dom.clone());
        let css_tokenizer = CssTokenizer::new(style);
        let cssom = CssParser::new(css_tokenizer).parse_stylesheet();
        LayoutView::new(dom, &cssom)
    }

    #[test]
    fn test_empty() {
        let layout_view = create_layout_view("".to_string());
        assert_eq!(None, layout_view.root())
    }

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind());

        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone().expect("root shoould exist").borrow().node_kind()
        );
    }

    #[test]
    fn test_text() {
        /*
         * TODO: なぜかテストが完了しない、エラーにもならない
         */
        let html = "<html><head></head><body>text</body></html>".to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root(); 
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone().expect("root should exist").borrow().node_kind()
        );

        let text = root.expect("root should exist").borrow().first_child();
        assert!(text.is_some());
        assert_eq!(
            LayoutObjectKind::Text,
            text.clone().expect("text node should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Text("text".to_string()),
            text.clone().expect("text node should exist").borrow().node_kind()
        );

    }

    #[test]
    fn test_display_none() {
        let html = "<html><head><style>body{display: none;}</style></head><body></body></html>".to_string();
        let layout_view = create_layout_view(html);

        assert_eq!(None, layout_view.root());
    }

    #[test]
    fn test_hidden_class() {

        /*
         * TODO: なぜかテストが完了しない、エラーにもならない
         */
        
        let html = r#"<html>
<head>
<style>
  .hidden {
    display: none;
  }
</style>
</head>
<body>
  <a class="hidden">link1</a>
  <p></p>
  <p class="hidden"><a>link2</a></p>
</body>
</html>"#
            .to_string();
        let layout_view = create_layout_view(html);

        let root = layout_view.root();
        assert!(root.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            root.clone().expect("root should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("body", Vec::new())),
            root.clone()
                .expect("root should exist")
                .borrow()
                .node_kind()
        );

        let p = root.expect("root should exist").borrow().first_child();
        assert!(p.is_some());
        assert_eq!(
            LayoutObjectKind::Block,
            p.clone().expect("p node should exist").borrow().kind()
        );
        assert_eq!(
            NodeKind::Element(Element::new("p", Vec::new())),
            p.clone().expect("p node should exist").borrow().node_kind()
        );

        assert!(p
            .clone()
            .expect("p node should exist")
            .borrow()
            .first_child()
            .is_none());
        assert!(p
            .expect("p node should exist")
            .borrow()
            .next_sibling()
            .is_none());
    }

    // TODO: css の　テストを追加する
}