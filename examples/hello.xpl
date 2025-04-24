<program name="hello" include="math.xpl" version="1.0">
  <description>hello world program</description>
  <function name="main">
	<body>
	  <var name="result" type="int"/>
	  <assign var="result">
	    <add>
	      <param> 5 </param>
	      <param> 3 </param>
		</add>
	  </assign>
	  <print> "Hello, World!" </print>
	  <print> "The result of 5 + 3 is: " </print>
	  <print> result </print>
	</body>
  </function>
</program>
