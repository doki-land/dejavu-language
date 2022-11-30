impl<% self.lifetime() %> core::fmt::Display for <% self.class_name() %><% self.lifetime() %> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
<% for node in self.statements %>
<% match node %>
    <% case DejavuStatement::Text(s) %>
        <% s %>
    <% case DejavuStatement::Branches(s) %>
        <% s %>
<% end match %>
<% end for %>
        Ok(())
    }
}
